use casper_types::{bytesrepr::Bytes, runtime_args, ContractHash, RuntimeArgs, SecretKey};
use casper_client::{types::{DeployBuilder, ExecutableDeployItem}};

async fn submit_batch_to_local_net(proof: Proof){
    let contract_hash: ContractHash = ContractHash::from_formatted_str("contract-cf2e4aa0ea2bb692a3b3decd89b241bc3e3b326b6e964333de7466d3891de29b").unwrap();
    let runtime_args = runtime_args! {"risc0_receipt" => Bytes::from(serde_json_wasm::to_vec(&proof).unwrap())};
    let deposit_session: ExecutableDeployItem = ExecutableDeployItem::StoredContractByHash { hash: contract_hash, entry_point: "submit_batch".to_string(), args: runtime_args };
    let deploy = DeployBuilder::new("cspr-dev-cctl", deposit_session, &SecretKey::from_file("/Users/chef/Desktop/secret_key.pem").unwrap()).with_standard_payment(1000_000_000_000u64).build().unwrap();
    let result = casper_client::put_deploy(
        casper_client::JsonRpcId::String("0".to_string()), 
        "http://127.0.0.1:11101", 
        casper_client::Verbosity::Low,
        deploy
    ).await.unwrap();
    println!("Transaction Result: {:?}", &result);
}

#[tokio::test]
async fn test_submit_batch(){
    let proof: Proof = generate_proof();
    submit_batch_to_local_net(proof).await;
}

fn main(){

}

fn generate_proof() -> Proof {
    extern crate alloc;
    use alloc::rc::Rc;
    use kairos_circuit_logic::{
        account_trie::Account, account_trie::AccountTrie, transactions::*, ProofInputs,
    };
    use kairos_trie::{stored::memory_db::MemoryDb, DigestHasher, TrieRoot};
    use methods::{PROVE_BATCH_ELF, PROVE_BATCH_ID};
    let alice_public_key = "alice_public_key".as_bytes().to_vec();
    //let bob_public_key = "bob_public_key".as_bytes().to_vec();
    let batch = vec![
        KairosTransaction::Deposit(L1Deposit {
            recipient: alice_public_key.clone(),
            amount: 10,
        }),
        /*KairosTransaction::Transfer(Signed {
            public_key: alice_public_key.clone(),
            transaction: Transfer {
                recipient: bob_public_key.clone(),
                amount: 5,
            },
            nonce: 0,
        }),
        KairosTransaction::Withdraw(Signed {
            public_key: alice_public_key.clone(),
            transaction: Withdraw { amount: 5 },
            nonce: 1,
        }),*/
    ];

    let db = Rc::new(MemoryDb::<Account>::empty());
    let prior_root_hash = TrieRoot::default();

    // the Trie is constructed from the current state of the DB.
    // keep in mind that the Trie, other than DeltaTree, stores Accounts
    // the entire DB state is used to construct a Snapshot for each proof.
    let mut account_trie = AccountTrie::new_try_from_db(db.clone(), prior_root_hash)
        .expect("Failed to create account trie");
    account_trie
        .apply_batch(batch.iter().cloned())
        .expect("Failed to apply batch");

    account_trie
        .txn
        .commit(&mut DigestHasher::<sha2::Sha256>::default())
        .expect("Failed to commit transaction");

    let trie_snapshot = account_trie.txn.build_initial_snapshot();

    let proof_inputs = ProofInputs {
        transactions: batch.into_boxed_slice(),
        trie_snapshot,
    };

    let env = risc0_zkvm::ExecutorEnv::builder()
        .write(&proof_inputs)
        .map_err(|e| format!("Error in ExecutorEnv builder write: {e}"))
        .unwrap()
        .build()
        .map_err(|e| format!("Error in ExecutorEnv builder build: {e}"))
        .unwrap();

    let receipt = risc0_zkvm::default_prover()
        .prove(env, PROVE_BATCH_ELF)
        .map_err(|e| format!("Error in risc0_zkvm prove: {e}"))
        .unwrap();

    receipt
        .verify(PROVE_BATCH_ID)
        .expect("Failed to verify proof!");

    Proof {
        receipt,
        program_id: PROVE_BATCH_ID,
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Proof {
    pub receipt: risc0_zkvm::Receipt,
    pub program_id: [u32; 8],
}
