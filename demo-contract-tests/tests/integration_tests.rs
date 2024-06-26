mod test_fixture;
#[cfg(test)]
mod tests {
    use crate::test_fixture::TestContext;
    use casper_types::U512;
    use kairos_verifier_risc0_lib::verifier::verify_execution;

    #[test]
    fn should_install_contract() {
        let _fixture = TestContext::new(None);
    }

    #[test]
    fn test_deposit_succeeds() {
        let mut fixture = TestContext::new(None);

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
        let mut fixture = TestContext::new(None);

        let user = fixture.create_funded_user();
        let amount = U512::from(100000000000u64);
        fixture.deposit_succeeds(user, amount);

        fixture.transfer_from_contract_purse_to_user_fails(user, amount)
    }

    #[test]
    fn test_transfer_from_contract_purse_to_admin_fails() {
        let mut fixture = TestContext::new(None);

        let user = fixture.create_funded_user();
        let amount = U512::from(100000000000u64);
        fixture.deposit_succeeds(user, amount);

        fixture.transfer_from_contract_purse_to_user_fails(fixture.admin, amount)
    }

    #[test]
    fn test_transfer_from_contract_purse_by_uref_to_user_fails() {
        let mut fixture = TestContext::new(None);
        let user = fixture.create_funded_user();
        let amount = U512::from(100000000000u64);
        fixture.deposit_succeeds(user, amount);

        fixture.transfer_from_contract_purse_by_uref_to_user_fails(user, amount)
    }

    #[test]
    fn test_transfer_from_contract_purse_by_uref_to_admin_fails() {
        let mut fixture = TestContext::new(None);
        let user = fixture.create_funded_user();
        let amount = U512::from(100000000000u64);
        fixture.deposit_succeeds(user, amount);
        fixture.transfer_from_contract_purse_by_uref_to_user_fails(fixture.admin, amount)
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
        // submit proofs to contract
        fixture.submit_proof_to_contract(fixture.admin, receipt0.to_vec());
        fixture.submit_proof_to_contract(fixture.admin, receipt1.to_vec());
    }

    // TODO some more real larger batches fail with code unreachable in the contract.
    // They verify fine outside the contract, so I suspect they use too much gas.
    #[allow(dead_code)]
    fn submit_batch_to_contract(receipt: &[u8]) {
        // precheck proofs before contract tests that are hard to debug
        let proof_outputs =
            verify_execution(&serde_json_wasm::from_slice(receipt).unwrap()).unwrap();

        eprintln!("{:?}", proof_outputs);

        let mut fixture = TestContext::new(proof_outputs.pre_batch_trie_root);
        fixture.submit_proof_to_contract(fixture.admin, receipt.to_vec())
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
