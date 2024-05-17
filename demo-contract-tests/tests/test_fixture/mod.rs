use casper_engine_test_support::{
    ExecuteRequestBuilder, WasmTestBuilder, ARG_AMOUNT, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{
    account::AccountHash,
    crypto::{PublicKey, SecretKey},
    runtime_args,
    system::{handle_payment::ARG_TARGET, mint::ARG_ID},
    RuntimeArgs, U512,
};
use std::path::{Path, PathBuf};

use casper_engine_test_support::{InMemoryWasmTestBuilder, PRODUCTION_RUN_GENESIS_REQUEST};
use casper_types::{ContractHash, URef};
use std::env;

pub const ADMIN_SECRET_KEY: [u8; 32] = [1u8; 32];

#[derive(Default)]
pub struct TestContext {
    builder: InMemoryWasmTestBuilder,
    pub admin: AccountHash,
    contract_hash: ContractHash,
    contract_purse: URef,
}

fn get_wasm_directory() -> PathBuf {
    // Environment variable or default path.
    let base_path = if let Ok(custom_path) = env::var("PATH_TO_WASM_BINARIES") {
        PathBuf::from(custom_path)
    } else if cfg!(debug_assertions) {
        PathBuf::from("../kairos-contracts/target/wasm32-unknown-unknown/debug/")
    } else {
        PathBuf::from("../kairos-contracts/target/wasm32-unknown-unknown/release/")
    };
    if !base_path.exists() {
        panic!("WASM directory does not exist: {}", base_path.display());
    }

    // TODO: Ensure all WASM files are optimized
    //optimize_wasm_files(&base_path)?;

    base_path
}

impl TestContext {
    pub fn new() -> TestContext {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST);

        let admin = create_funded_account_for_secret_key_bytes(&mut builder, ADMIN_SECRET_KEY);
        let contract_path = get_wasm_directory().join("demo-contract-optimized.wasm");
        run_session_with_args(&mut builder, &contract_path, admin, runtime_args! {});

        let contract_hash = builder
            .get_expected_account(admin)
            .named_keys()
            .get("kairos_contract_hash")
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash");

        let contract = builder
            .get_contract(contract_hash)
            .expect("should have contract");
        let contract_purse = *contract
            .named_keys()
            .get("kairos_deposit_purse")
            .expect("Key not found")
            .as_uref()
            .unwrap();

        TestContext {
            builder,
            admin,
            contract_hash,
            contract_purse,
        }
    }

    pub fn create_funded_user(&mut self) -> AccountHash {
        let mut random_secret_key: [u8; 32] = rand::random();
        while random_secret_key == ADMIN_SECRET_KEY {
            random_secret_key = rand::random();
        }
        create_funded_account_for_secret_key_bytes(&mut self.builder, random_secret_key)
    }

    pub fn get_user_balance(&mut self, user: AccountHash) -> U512 {
        let user_uref = self.builder.get_expected_account(user).main_purse();
        self.builder.get_purse_balance(user_uref)
    }

    pub fn get_contract_balance(&mut self) -> U512 {
        self.builder.get_purse_balance(self.contract_purse)
    }

    pub fn deposit_succeeds(&mut self, depositor: AccountHash, amount: U512) {
        let deposit_session_path = get_wasm_directory().join("deposit-session-optimized.wasm");
        let session_args = runtime_args! {
            "amount" => amount,
            "demo_contract" => self.contract_hash
        };
        run_session_with_args(
            &mut self.builder,
            deposit_session_path.as_path(),
            depositor,
            session_args,
        );
        self.builder.expect_success();
    }

    pub fn transfer_from_contract_purse_to_user_fails(
        &mut self,
        receiver: AccountHash,
        amount: U512,
    ) {
        let session_args = runtime_args! {
            "amount" => amount,
            "demo_contract" => self.contract_hash
        };
        let malicious_session_path = get_wasm_directory().join("malicious-session-optimized.wasm");
        run_session_with_args(
            &mut self.builder,
            malicious_session_path.as_path(),
            receiver,
            session_args,
        );
        self.builder.expect_failure();
    }
    pub fn transfer_from_contract_purse_by_uref_to_user_fails(
        &mut self,
        receiver: AccountHash,
        amount: U512,
    ) {
        let session_args = runtime_args! {
            "amount" => amount,
            "demo_contract" => self.contract_hash,
            "purse_uref" => self.contract_purse
        };
        let malicious_reader_session_path =
            get_wasm_directory().join("malicious-reader-optimized.wasm");
        run_session_with_args(
            &mut self.builder,
            malicious_reader_session_path.as_path(),
            receiver,
            session_args,
        );
        self.builder.expect_failure();
    }
}

pub fn run_session_with_args(
    builder: &mut WasmTestBuilder<InMemoryGlobalState>,
    session_wasm_path: &Path,
    user: AccountHash,
    runtime_args: RuntimeArgs,
) {
    let session_request =
        ExecuteRequestBuilder::standard(user, session_wasm_path.to_str().unwrap(), runtime_args)
            .build();
    builder.exec(session_request).commit();
}

/// Creates a funded account for the given ed25519 secret key in bytes
/// It panics if the passed secret key bytes cannot be read
pub fn create_funded_account_for_secret_key_bytes(
    builder: &mut WasmTestBuilder<InMemoryGlobalState>,
    account_secret_key_bytes: [u8; 32],
) -> AccountHash {
    let account_secret_key = SecretKey::ed25519_from_bytes(account_secret_key_bytes).unwrap();
    let account_public_key = PublicKey::from(&account_secret_key);
    let account_hash = account_public_key.to_account_hash();
    let transfer = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            ARG_AMOUNT => DEFAULT_ACCOUNT_INITIAL_BALANCE / 10_u64,
            ARG_TARGET => account_hash,
            ARG_ID => Option::<u64>::None,
        },
    )
    .build();
    builder.exec(transfer).expect_success().commit();
    account_hash
}
