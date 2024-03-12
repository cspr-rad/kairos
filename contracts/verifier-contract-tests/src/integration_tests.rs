mod test_fixture;
#[cfg(test)]
mod tests {
    use crate::test_fixture::TestContext;
    use casper_types::{account::AccountHash, U512};

    /*
    #[test]
    fn should_install() {
        let mut fixture: TestContext = TestContext::new();
        fixture.install(fixture.account_1);
    }*/
    #[test]
    fn should_prove_batch(){
        let mut fixture: TestContext = TestContext::new();
        fixture.install(fixture.account_1);
        fixture.submit_batch(fixture.account_1);
    }
}