mod test_fixture;
#[cfg(test)]
mod tests {
    use crate::test_fixture::{run_session_with_args, TestContext};
    use casper_types::{runtime_args, RuntimeArgs, U512};
    use std::path::Path;

    #[test]
    fn should_install_deposit_contract() {
        let _fixture = TestContext::new();
    }

    #[test]
    fn deposit_into_purse() {
        let TestContext {
            mut builder,
            user_1,
            contract_hash,
            contract_purse,
        } = TestContext::new();

        let user_uref = builder.get_expected_account(user_1).main_purse();
        let user_balance_before = builder.get_purse_balance(user_uref);

        // check that the contract balance is zero before depositing
        let contract_balance_before = builder.get_purse_balance(contract_purse);
        assert_eq!(contract_balance_before, U512::zero());

        // user_1 deposits the deposit_amount
        let deposit_session_path =
            Path::new(env!("PATH_TO_WASM_BINARIES")).join("deposit-session-optimized.wasm");
        let deposit_amount = U512::from(100000000000u64);
        let session_args = runtime_args! {
            "amount" => deposit_amount,
            "demo_contract" => contract_hash
        };
        run_session_with_args(
            &mut builder,
            deposit_session_path.as_path(),
            user_1,
            session_args,
        );
        builder.expect_success();

        // the contract balance should afterward equal to the deposit_amount
        let contract_balance_after = builder.get_purse_balance(contract_purse);
        assert_eq!(contract_balance_after, deposit_amount);

        let user_balance_after = builder.get_purse_balance(user_uref);
        assert!(user_balance_after <= user_balance_before - deposit_amount);
    }

    // see malicious-session
    #[test]
    fn run_malicious_session() {
        let TestContext {
            mut builder,
            user_1,
            contract_hash,
            ..
        } = TestContext::new();

        let deposit_session_path = std::path::Path::new(env!("PATH_TO_WASM_BINARIES"))
            .join("deposit-session-optimized.wasm");
        let session_args = runtime_args! {
            "amount" => U512::from(100000000000u64),
            "demo_contract" => contract_hash
        };
        run_session_with_args(
            &mut builder,
            deposit_session_path.as_path(),
            user_1,
            session_args.clone(),
        );
        builder.expect_success();

        let malicious_session_path = std::path::Path::new(env!("PATH_TO_WASM_BINARIES"))
            .join("malicious-session-optimized.wasm");
        run_session_with_args(
            &mut builder,
            malicious_session_path.as_path(),
            user_1,
            session_args,
        );
        builder.expect_failure();
    }

    // see malicious-reader
    #[test]
    fn run_malicious_reader() {
        let TestContext {
            mut builder,
            user_1,
            contract_hash,
            contract_purse,
        } = TestContext::new();

        let deposit_session_path = std::path::Path::new(env!("PATH_TO_WASM_BINARIES"))
            .join("deposit-session-optimized.wasm");
        let session_args = runtime_args! {
            "amount" => U512::from(100000000000u64),
            "demo_contract" => contract_hash
        };
        run_session_with_args(
            &mut builder,
            deposit_session_path.as_path(),
            user_1,
            session_args,
        );
        builder.expect_success();

        let deposit_amount = U512::from(100000000000u64);
        let session_args = runtime_args! {
            "amount" => deposit_amount,
            "demo_contract" => contract_hash,
            "purse_uref" => contract_purse
        };
        let malicious_reader_session_path = std::path::Path::new(env!("PATH_TO_WASM_BINARIES"))
            .join("malicious-reader-optimized.wasm");
        run_session_with_args(
            &mut builder,
            malicious_reader_session_path.as_path(),
            user_1,
            session_args,
        );
        builder.expect_failure();
    }
}
