mod test_fixture;
#[cfg(test)]
mod tests {
    use crate::test_fixture::TestContext;
    use casper_types::U512;
    use risc0_zkvm::serde::to_vec;

    #[test]
    fn should_install_contract() {
        let _fixture = TestContext::new();
    }

    #[test]
    fn test_deposit_succeeds() {
        let mut fixture = TestContext::new();

        let user = fixture.create_funded_user();
        let user_balance_before = fixture.get_user_balance(user);

        // check that the contract balance is zero before depositing
        let deposit_amount = U512::from(100000000000u64);
        let contract_balance_before = fixture.get_contract_balance();
        assert_eq!(contract_balance_before, U512::zero());

        // user_1 deposits the deposit_amount
        fixture.deposit_succeeds(user, deposit_amount);

        // the contract balance should afterward equal to the deposit_amount
        let contract_balance_after = fixture.get_contract_balance();
        assert_eq!(contract_balance_after, deposit_amount);

        let user_balance_after = fixture.get_user_balance(user);
        assert!(user_balance_after <= user_balance_before - deposit_amount);
    }

    #[test]
    fn test_transfer_from_contract_purse_to_user_fails() {
        let mut fixture = TestContext::new();

        let user = fixture.create_funded_user();
        let amount = U512::from(100000000000u64);
        fixture.deposit_succeeds(user, amount);

        fixture.transfer_from_contract_purse_to_user_fails(user, amount)
    }

    #[test]
    fn test_transfer_from_contract_purse_to_admin_fails() {
        let mut fixture = TestContext::new();

        let user = fixture.create_funded_user();
        let amount = U512::from(100000000000u64);
        fixture.deposit_succeeds(user, amount);

        fixture.transfer_from_contract_purse_to_user_fails(fixture.admin, amount)
    }

    #[test]
    fn test_transfer_from_contract_purse_by_uref_to_user_fails() {
        let mut fixture = TestContext::new();
        let user = fixture.create_funded_user();
        let amount = U512::from(100000000000u64);
        fixture.deposit_succeeds(user, amount);

        fixture.transfer_from_contract_purse_by_uref_to_user_fails(user, amount)
    }

    #[test]
    fn test_transfer_from_contract_purse_by_uref_to_admin_fails() {
        let mut fixture = TestContext::new();
        let user = fixture.create_funded_user();
        let amount = U512::from(100000000000u64);
        fixture.deposit_succeeds(user, amount);
        fixture.transfer_from_contract_purse_by_uref_to_user_fails(fixture.admin, amount)
    }

    #[test]
    fn submit_batch_to_contract() {
        let mut fixture = TestContext::new();
        let proof: Proof = generate_proof();
        fixture.submit_proof_to_contract(fixture.admin, serde_json_wasm::to_vec(&proof).unwrap())
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
}
