mod test_fixture;
#[cfg(test)]
mod tests {
    use crate::test_fixture::TestContext;
    use casper_types::{
        account::{self, AccountHash},
        U512,
    };

    #[test]
    fn should_install() {
        let mut fixture: TestContext = TestContext::new();
        fixture.install(fixture.account_1);
    }

    fn setup() -> (TestContext, AccountHash) {
        let mut fixture: TestContext = TestContext::new();
        let installer = fixture.account_1;
        return (fixture, installer);
    }

    fn create_contract_purse(fixture: &mut TestContext, installer: AccountHash) {
        fixture.install(installer);
        fixture.create_contract_purse(fixture.account_1, installer);
    }

    #[test]
    fn deposit_into_purse() {
        let (mut fixture, installer) = setup();
        create_contract_purse(&mut fixture, installer);
        fixture.run_deposit_session(U512::from(100000000000u64), installer);
    }

    #[test]
    fn withdrawal_from_purse() {
        let (mut fixture, installer) = setup();
        create_contract_purse(&mut fixture, installer);
        fixture.run_deposit_session(U512::from(100000000000u64), installer);
        //fixture.get_contract_purse_balance(installer);
        fixture.run_withdrawal_session(fixture.account_1, U512::from(100000000000u64), installer)
    }

    // attack vector 1: session code attempts to empty purse
    #[test]
    fn run_malicious_session() {
        let (mut fixture, installer) = setup();
        create_contract_purse(&mut fixture, installer);
        fixture.run_deposit_session(U512::from(100000000000u64), installer);
        fixture.run_malicious_session(fixture.account_2, U512::from(100000000000u64), installer);
    }

    // attack vector 2: smart contract access control
    #[test]
    fn run_malicious_contract() {
        let (mut fixture, installer) = setup();
        create_contract_purse(&mut fixture, installer);
        fixture.run_deposit_session(U512::from(100000000000u64), installer);
        fixture.run_malicious_withdrawal_session(
            fixture.account_2,
            U512::from(100000000000u64),
            installer,
        );
    }

    // attack vector 3: purse URef in contract named keys
    #[test]
    fn run_malicious_reader() {
        let (mut fixture, installer) = setup();
        create_contract_purse(&mut fixture, installer);
        fixture.run_deposit_session(U512::from(100000000000u64), installer);
        let deposit_purse_uref = fixture.get_contract_purse_uref(installer);
        fixture.run_malicious_reader_session(
            fixture.account_2,
            U512::from(100000000000u64),
            installer,
            deposit_purse_uref,
        );
    }
}
