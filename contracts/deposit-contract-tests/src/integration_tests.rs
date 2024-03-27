mod test_fixture;
#[cfg(test)]
mod tests {
    use crate::test_fixture::TestContext;
    use casper_types::{account::AccountHash, U512};

    #[test]
    fn should_install() {
        let mut fixture: TestContext = TestContext::new();
        fixture.install(fixture.account_1);
    }

    fn setup() -> (TestContext, AccountHash) {
        let fixture: TestContext = TestContext::new();
        let installer = fixture.account_1;
        return (fixture, installer);
    }

    #[test]
    fn deposit_into_purse() {
        let (mut fixture, installer) = setup();
        fixture.install(installer);
        fixture.run_deposit_session(U512::from(100000000000u64), installer);
    }

    // see malicious-session
    #[test]
    fn run_malicious_session() {
        let (mut fixture, installer) = setup();
        fixture.install(installer);
        fixture.run_deposit_session(U512::from(100000000000u64), installer);
        fixture.run_malicious_session(fixture.account_2, U512::from(100000000000u64), installer);
    }

    // see malicious-reader
    #[test]
    fn run_malicious_reader() {
        let (mut fixture, installer) = setup();
        fixture.install(installer);
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
