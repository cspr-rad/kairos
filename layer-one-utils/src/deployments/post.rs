use casper_client::{rpcs::results::PutDeployResult, types::{Deploy, DeployBuilder, ExecutableDeployItem, Timestamp}, JsonRpcId};
use casper_types::{bytesrepr::{Bytes, ToBytes}, crypto::SecretKey, runtime_args, ContractHash, RuntimeArgs};
use crate::constants::{CCTL_DEFAULT_NODE_ADDRESS, CCTL_DEFAULT_NODE_RPC_PORT, DEFAULT_CHAIN_NAME, DEFAULT_PAYMENT_AMOUNT, SECRET_KEY_PATH, VERIFIER_CONTRACT_HASH};
use std::fs;

pub async fn submit_delta_tree_batch(
    node_address: &str,
    rpc_port: &str,
    secret_key_path: &str,
    chain_name: &str,
    contract: &str,
    payload: Bytes,
){
    let session: ExecutableDeployItem = ExecutableDeployItem::StoredContractByHash { 
        hash: ContractHash::from_formatted_str(contract).unwrap(), 
        entry_point: "submit_delta_tree_batch".to_string(), 
        args: runtime_args!{
            "proof" => payload
        }
    };
    let secret_key_bytes: Vec<u8> = fs::read(secret_key_path).unwrap();
    let secret_key: SecretKey = SecretKey::from_pem(secret_key_bytes.clone()).unwrap();
    
    let deploy: Deploy = DeployBuilder::new(
        chain_name,
        session
    ).with_timestamp(Timestamp::now()).with_standard_payment(DEFAULT_PAYMENT_AMOUNT).with_secret_key(&secret_key).build().unwrap();

    let result = casper_client::put_deploy(
        JsonRpcId::String(rpc_port.to_string()), 
        node_address, 
        casper_client::Verbosity::Low,
        deploy
    ).await.unwrap();
    
    println!("Deploy result: {:?}", &result);
}

#[tokio::test]
async fn test_submit_delta_tree_batch(){
    use kairos_risc0_types::{KairosDeltaTree, hash_bytes, TransactionBatch, Deposit, Withdrawal, Transfer, RiscZeroProof, CircuitJournal, CircuitArgs};
    use risc0_zkvm::{default_prover, Receipt, ExecutorEnv};
    use methods::{
        NATIVE_CSPR_TX_ELF, NATIVE_CSPR_TX_ID
    };
    let mut tree: KairosDeltaTree = KairosDeltaTree{
        zero_node: hash_bytes(vec![0;32]),
        zero_levels: Vec::new(),
        filled: vec![vec![], vec![], vec![], vec![], vec![]],
        root: None,
        index: 0,
        depth: 5
    };
    tree.calculate_zero_levels();
    let transfers: Vec<Transfer> = vec![/*Transfer{
        sender: Key::from_formatted_str("account-hash-32da6919b3a0a9be4bc5b38fa74de98f90dc43924bf17e73f6635992f110f011").unwrap(),
        recipient: Key::from_formatted_str("account-hash-32da6919b3a0a9be4bc5b38fa74de98f90dc43924bf17e73f6635992f110f011").unwrap(),
        amount: U512::zero(),
        timestamp: "SOME_TIMESTAMP".to_string(),
        signature: vec![],
        processed: false,
        nonce: 0u64
    }*/];
    let deposits: Vec<Deposit> = vec![];
    let withdrawals: Vec<Withdrawal> = vec![];
    let batch: TransactionBatch = TransactionBatch{
        transfers,
        deposits, 
        withdrawals
    };
    let proof: RiscZeroProof = prove_batch(tree, batch);
    let bincode_serialized_proof: Vec<u8> = bincode::serialize(&proof).expect("Failed to serialize proof!");
    let cl_proof: Bytes = Bytes::from(bincode_serialized_proof);
    submit_delta_tree_batch(CCTL_DEFAULT_NODE_ADDRESS, CCTL_DEFAULT_NODE_RPC_PORT, SECRET_KEY_PATH, DEFAULT_CHAIN_NAME, VERIFIER_CONTRACT_HASH, cl_proof).await;

    pub fn prove_batch(tree: KairosDeltaTree, batch: TransactionBatch) -> RiscZeroProof{
        let inputs = CircuitArgs{
            tree,
            batch
        };
        let env = ExecutorEnv::builder()
        .write(&inputs)
        .unwrap()
        .build()
        .unwrap();
    
        let prover = default_prover();
        let receipt = prover.prove(env, NATIVE_CSPR_TX_ELF).unwrap();
        receipt.verify(NATIVE_CSPR_TX_ID).expect("Failed to verify proof!");
        RiscZeroProof{
            receipt_serialized: bincode::serialize(&receipt).unwrap(),
            program_id: NATIVE_CSPR_TX_ID.to_vec()
        }
    }
}