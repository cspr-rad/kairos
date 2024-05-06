// mod test_fixture;
// #[cfg(test)]
// mod tests {
//     use crate::test_fixture::TestContext;
//     use casper_types::{account::AccountHash, U512};

//     #[test]
//     fn should_install_deposit_contract() {
//         let mut fixture: TestContext = TestContext::new();
//         fixture.install_demo_contract(fixture.account_1);
//     }

//     fn setup() -> (TestContext, AccountHash, AccountHash) {
//         let fixture: TestContext = TestContext::new();
//         let installer = fixture.account_1;
//         let user = fixture.account_2;
//         (fixture, installer, user)
//     }

//     #[test]
//     fn deposit_into_purse() {
//         let deposit_amount: U512 = U512::from(100000000000u64);
//         let (mut fixture, installer, user) = setup();
//         fixture.install_demo_contract(installer);

//         let user_purse_uref = fixture.get_account_purse_uref(user);
//         let user_balance_before = fixture.builder.get_purse_balance(user_purse_uref);

//         let contract_balance_before = fixture.get_contract_purse_balance(installer);
//         assert_eq!(contract_balance_before, U512::zero());

//         fixture.run_deposit_session(deposit_amount, installer, user);

//         let contract_balance_after = fixture.get_contract_purse_balance(installer);
//         assert_eq!(contract_balance_after, deposit_amount);

//         let user_balance_after = fixture.builder.get_purse_balance(user_purse_uref);
//         assert!(user_balance_after <= user_balance_before - deposit_amount);
//     }

//     // see malicious-session
//     #[test]
//     fn run_malicious_session() {
//         let (mut fixture, installer, user) = setup();
//         fixture.install_demo_contract(installer);
//         fixture.run_deposit_session(U512::from(100000000000u64), installer, user);
//         fixture.run_malicious_session(fixture.account_2, U512::from(100000000000u64), installer);
//     }

//     // see malicious-reader
//     #[test]
//     fn run_malicious_reader() {
//         let (mut fixture, installer, user) = setup();
//         fixture.install_demo_contract(installer);
//         fixture.run_deposit_session(U512::from(100000000000u64), installer, user);
//         let deposit_purse_uref = fixture.get_contract_purse_uref(installer);
//         fixture.run_malicious_reader_session(
//             fixture.account_2,
//             U512::from(100000000000u64),
//             installer,
//             deposit_purse_uref,
//         );
//     }
// }
