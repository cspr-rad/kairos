mod test_fixture;
#[cfg(test)]
mod tests {
    use crate::test_fixture::TestContext;
    use casper_types::U512;

    #[test]
    fn should_install_deposit_contract() {
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
    fn test_transfer_from_contract_purse_by_uref_to_user_should_fails() {
        let mut fixture = TestContext::new();
        let user = fixture.create_funded_user();
        let amount = U512::from(100000000000u64);
        fixture.deposit_succeeds(user, amount);

        fixture.transfer_from_contract_purse_by_uref_to_user_fails(user, amount)
    }

    #[test]
    fn test_transfer_from_contract_purse_by_uref_to_admin_should_fails() {
        let mut fixture = TestContext::new();
        let user = fixture.create_funded_user();
        let amount = U512::from(100000000000u64);
        fixture.deposit_succeeds(user, amount);

        fixture.transfer_from_contract_purse_by_uref_to_user_fails(fixture.admin, amount)
    }
}
