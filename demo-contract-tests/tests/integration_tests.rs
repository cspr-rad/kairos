mod test_fixture;
#[cfg(test)]
mod tests {
    use crate::test_fixture::TestContext;
    use casper_types::{PublicKey, SecretKey, U512};
    use kairos_verifier_risc0_lib::verifier::verify_execution;

    #[test]
    fn should_install_contract() {
        let _fixture = TestContext::new(None);
    }

    #[test]
    fn test_deposit_succeeds() {
        let mut fixture = TestContext::new(None);

        let user = fixture.create_funded_user();
        let user_account_hash = user.to_account_hash();
        let user_balance_before = fixture.get_user_balance(user_account_hash);

        // check that the contract balance is zero before depositing
        let deposit_amount = U512::from(100000000000u64);
        let contract_balance_before = fixture.get_contract_balance();
        assert_eq!(contract_balance_before, U512::zero());

        // user_1 deposits the deposit_amount
        fixture.deposit_succeeds(user, deposit_amount);

        // the contract balance should afterward equal to the deposit_amount
        let contract_balance_after = fixture.get_contract_balance();
        assert_eq!(contract_balance_after, deposit_amount);

        let user_balance_after = fixture.get_user_balance(user_account_hash);
        assert!(user_balance_after <= user_balance_before - deposit_amount);
    }

    #[test]
    fn test_transfer_from_contract_purse_to_user_fails() {
        let mut fixture = TestContext::new(None);

        let user = fixture.create_funded_user();
        let user_account_hash = user.to_account_hash();
        let amount = U512::from(100000000000u64);
        fixture.deposit_succeeds(user, amount);

        fixture.transfer_from_contract_purse_to_user_fails(user_account_hash, amount)
    }

    #[test]
    fn test_transfer_from_contract_purse_to_admin_fails() {
        let mut fixture = TestContext::new(None);

        let user = fixture.create_funded_user();
        let amount = U512::from(100000000000u64);
        fixture.deposit_succeeds(user.clone(), amount);

        fixture.transfer_from_contract_purse_to_user_fails(fixture.admin, amount)
    }

    #[test]
    fn test_transfer_from_contract_purse_by_uref_to_user_fails() {
        let mut fixture = TestContext::new(None);
        let user = fixture.create_funded_user();
        let user_account_hash = user.to_account_hash();
        let amount = U512::from(100000000000u64);
        fixture.deposit_succeeds(user.clone(), amount);

        fixture.transfer_from_contract_purse_by_uref_to_user_fails(user_account_hash, amount)
    }

    #[test]
    fn test_transfer_from_contract_purse_by_uref_to_admin_fails() {
        let mut fixture = TestContext::new(None);
        let user = fixture.create_funded_user();
        let amount = U512::from(100000000000u64);
        fixture.deposit_succeeds(user, amount);
        fixture.transfer_from_contract_purse_by_uref_to_user_fails(fixture.admin, amount)
    }

    /// Delete this test when proof-from-server.json goes out of date.
    #[test]
    fn submit_batch_to_contract_proof_from_server() {
        let receipt = include_bytes!("testdata/proof-from-server.json");

        // precheck proofs before contract tests that are hard to debug
        let proof_outputs =
            verify_execution(&serde_json_wasm::from_slice(receipt).unwrap()).unwrap();
        assert_eq!(proof_outputs.pre_batch_trie_root, None);

        let mut fixture = TestContext::new(None);

        // must match the key in the receipt simple_batches_0
        let user_1_secret_key =
            SecretKey::from_pem(include_str!("../../testdata/users/user-1/secret_key.pem"))
                .unwrap();

        let user_1_public_key = fixture.create_funded_account_for_secret_key(user_1_secret_key);
        let user_1_account_hash = user_1_public_key.to_account_hash();

        let user_1_pre_deposit_bal = fixture.get_user_balance(user_1_account_hash);
        fixture.deposit_succeeds(user_1_public_key.clone(), U512::from(500u64));
        fixture.deposit_succeeds(user_1_public_key, U512::from(500u64));
        let user_1_post_deposit_bal = fixture.get_user_balance(user_1_account_hash);

        // submit proof to contract
        fixture.submit_proof_to_contract_expect_success(fixture.admin, receipt.to_vec());

        assert!(user_1_post_deposit_bal <= user_1_pre_deposit_bal - U512::from(5u64));
    }

    #[test]
    fn submit_batch_to_contract_simple() {
        let receipt0 = include_bytes!("testdata/test_prove_simple_batches_0.json");
        let receipt1 = include_bytes!("testdata/test_prove_simple_batches_1.json");

        // precheck proofs before contract tests that are hard to debug
        let proof_outputs =
            verify_execution(&serde_json_wasm::from_slice(receipt0).unwrap()).unwrap();
        assert_eq!(proof_outputs.pre_batch_trie_root, None);
        verify_execution(&serde_json_wasm::from_slice(receipt1).unwrap()).unwrap();

        let mut fixture = TestContext::new(None);

        // must match the key in the receipt simple_batches_0
        let alice_secret_key =
            SecretKey::from_pem(include_str!("../../testdata/users/user-2/secret_key.pem"))
                .unwrap();

        let alice_public_key = fixture.create_funded_account_for_secret_key(alice_secret_key);
        let alice_account_hash = alice_public_key.to_account_hash();

        let bob_secret_key =
            SecretKey::from_pem(include_str!("../../testdata/users/user-3/secret_key.pem"))
                .unwrap();
        let bob_public_key = PublicKey::from(&bob_secret_key);
        let bob_account_hash = bob_public_key.to_account_hash();

        // must match the amount in the receipt simple_batches_0
        fixture.deposit_succeeds(alice_public_key, U512::from(10u64));
        let alice_post_deposit_bal = fixture.get_user_balance(alice_account_hash);

        // submit proofs to contract
        fixture.submit_proof_to_contract_expect_success(fixture.admin, receipt0.to_vec());

        let alice_post_batch_1_bal = fixture.get_user_balance(alice_account_hash);
        assert_eq!(
            alice_post_batch_1_bal,
            alice_post_deposit_bal + U512::from(5u64)
        );

        fixture.submit_proof_to_contract_expect_success(fixture.admin, receipt1.to_vec());

        // must match the logic in the kairos_prover simple_batches test.
        let bob_post_batch_2_bal = fixture.get_user_balance(bob_account_hash);
        assert_eq!(bob_post_batch_2_bal, U512::from(3u64));

        let alice_post_batch_2_bal = fixture.get_user_balance(alice_account_hash);
        assert_eq!(
            alice_post_batch_2_bal,
            alice_post_batch_1_bal + U512::from(2u64)
        );
    }

    // TODO some more real larger batches fail with code unreachable in the contract.
    // They verify fine outside the contract, so I suspect they use too much gas.
    fn submit_batch_to_contract(receipt: &[u8]) {
        // precheck proofs before contract tests that are hard to debug
        let proof_outputs =
            verify_execution(&serde_json_wasm::from_slice(receipt).unwrap()).unwrap();

        eprintln!("{:?}", proof_outputs);

        let mut fixture = TestContext::new(proof_outputs.pre_batch_trie_root);
        let api_err =
            fixture.submit_proof_to_contract_expect_api_err(fixture.admin, receipt.to_vec());

        // We expect error 201 which occurs after proof verification
        // when the proof outputs deposits are checked against the contract's deposits.
        //
        // Since we have not made any deposits on the l1 an error is expected.
        //
        // In the future it would be nice to make these prop test batches use real public keys so
        // we could make this test pass all the way through.
        assert_eq!(api_err, casper_types::ApiError::User(201));
    }

    #[test]
    fn submit_batch_to_contract_1() {
        let receipt =
            include_bytes!("testdata/proptest_prove_batches-proof-journal-517453938a5b4f3e.json");
        submit_batch_to_contract(receipt);
    }

    #[test]
    fn submit_batch_to_contract_2() {
        let receipt =
            include_bytes!("testdata/proptest_prove_batches-proof-journal-9a85f9117f0bf3a8.json");
        submit_batch_to_contract(receipt);
    }

    #[test]
    fn submit_batch_to_contract_3() {
        let receipt =
            include_bytes!("testdata/proptest_prove_batches-proof-journal-52945c21c49e8ca6.json");
        submit_batch_to_contract(receipt);
    }

    #[test]
    fn submit_batch_to_contract_4() {
        let receipt =
            include_bytes!("testdata/proptest_prove_batches-proof-journal-a53b58a9cf37e50e.json");
        submit_batch_to_contract(receipt);
    }
}
