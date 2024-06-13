mod test_fixture;
#[cfg(test)]
mod tests {
    use crate::test_fixture::TestContext;
    use casper_types::U512;
    use kairos_verifier_risc0_lib::verifier::verify_execution;

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
    fn submit_batch_to_contract_simple() {
        let mut fixture = TestContext::new();
        let receipt0 = include_bytes!("testdata/test_prove_simple_batches_0.json");
        let receipt1 = include_bytes!("testdata/test_prove_simple_batches_1.json");

        // precheck proofs before contract tests that are hard to debug
        let proof_outputs =
            verify_execution(&serde_json_wasm::from_slice(receipt0).unwrap()).unwrap();
        assert_eq!(proof_outputs.pre_batch_trie_root, None);
        verify_execution(&serde_json_wasm::from_slice(receipt1).unwrap()).unwrap();

        // submit proofs to contract
        fixture.submit_proof_to_contract(fixture.admin, receipt0.to_vec());
        fixture.submit_proof_to_contract(fixture.admin, receipt1.to_vec());
    }

    // TODO all these more real batches use too much gas
    // fn submit_batch_to_contract(receipt: &[u8]) {
    //     let mut fixture = TestContext::new();

    //     let receipt: Receipt = serde_json_wasm::from_slice(receipt).unwrap();
    //     verify_execution(&receipt).unwrap();

    //     fixture.submit_proof_to_contract(fixture.admin, serde_json_wasm::to_vec(&receipt).unwrap());
    // }

    // #[test]
    // fn submit_batch_to_contract_1() {
    //     let receipt =
    //         include_bytes!("testdata/proptest_prove_batches-proof-journal-c77eac1aed36d104.json");

    //     submit_batch_to_contract(receipt);
    // }
    // #[test]
    // fn submit_batch_to_contract_1() {
    //     let mut fixture = TestContext::new();
    //     let receipt =
    //         include_bytes!("testdata/proptest_prove_batches-proof-journal-c77eac1aed36d104.json");

    //     let receipt: Receipt = serde_json_wasm::from_slice(receipt).unwrap();

    //     fixture.submit_proof_to_contract(fixture.admin, serde_json_wasm::to_vec(&receipt).unwrap());
    // }

    // #[test]
    // fn submit_batch_to_contract_2() {
    //     let mut fixture = TestContext::new();
    //     let receipt =
    //         include_bytes!("testdata/proptest_prove_batches-proof-journal-7d8dadeda4c1eb1c.json");
    //     fixture.submit_proof_to_contract(fixture.admin, receipt.to_vec());
    // }

    // #[test]
    // fn submit_batch_to_contract_3() {
    //     let mut fixture = TestContext::new();
    //     let receipt =
    //         include_bytes!("testdata/proptest_prove_batches-proof-journal-3673e712f7cc58df.json");
    //     fixture.submit_proof_to_contract(fixture.admin, receipt.to_vec());
    // }
}
