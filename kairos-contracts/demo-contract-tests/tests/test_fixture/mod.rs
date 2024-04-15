mod utils;
use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, PRODUCTION_RUN_GENESIS_REQUEST,
};
use casper_types::{
    account::AccountHash, contracts::NamedKeys, runtime_args, ContractHash, Key, RuntimeArgs, URef,
    U512,
};
use lazy_static::lazy_static;
use utils::create_funded_dummy_account;
extern crate dotenvy;
use dotenvy::dotenv;
use std::env;

pub const ACCOUNT_USER_1: [u8; 32] = [1u8; 32];
pub const ACCOUNT_USER_2: [u8; 32] = [2u8; 32];
pub const ACCOUNT_USER_3: [u8; 32] = [3u8; 32];

// This defines a static variable for the path to WASM binaries
lazy_static! {
    static ref PATH_TO_WASM_BINARIES: String = {
        dotenv().ok();
        env::var("PATH_TO_WASM_BINARIES")
            .expect("Missing environment variable PATH_TO_WASM_BINARIES")
    };
}

pub struct TestContext {
    pub builder: InMemoryWasmTestBuilder,
    pub account_1: AccountHash,
    pub account_2: AccountHash,
    #[allow(dead_code)]
    pub account_3: AccountHash,
}

impl TestContext {
    pub fn new() -> TestContext {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST);
        let account_1 = create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));
        let account_2 = create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_2));
        let account_3 = create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_3));

        TestContext {
            builder,
            account_1,
            account_2,
            account_3,
        }
    }

    pub fn named_keys(&self, account: AccountHash) -> NamedKeys {
        self.builder
            .get_expected_account(account)
            .named_keys()
            .clone()
    }

    pub fn get_contract_named_key(
        &self,
        contract_name: &str,
        key_name: &str,
        account: AccountHash,
    ) -> Key {
        let contract_hash = self.contract_hash_from_named_keys(contract_name, account);
        *self
            .builder
            .get_contract(contract_hash)
            .expect("should have contract")
            .named_keys()
            .get(key_name)
            .expect("Key not found")
    }

    pub fn contract_hash_from_named_keys(
        &self,
        key_name: &str,
        account: AccountHash,
    ) -> ContractHash {
        self.named_keys(account)
            .get(key_name)
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash")
    }

    pub fn contract_hash(&self, name: &str, account: AccountHash) -> ContractHash {
        self.builder
            .get_expected_account(account)
            .named_keys()
            .get(name)
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash")
    }

    pub fn install_demo_contract(&mut self, admin: AccountHash) {
        let session_args = runtime_args! {};
        let install_contract_request = ExecuteRequestBuilder::standard(
            admin,
            &format!(
                "{}/{}",
                *PATH_TO_WASM_BINARIES, "demo-contract-optimized.wasm"
            ),
            session_args,
        )
        .build();
        self.builder
            .exec(install_contract_request)
            .expect_success()
            .commit();
    }

    pub fn get_contract_purse_uref(&self, account: AccountHash) -> URef {
        let seed_uref: URef = *self
            .get_contract_named_key("kairos_demo_contract", "kairos_deposit_purse", account)
            .as_uref()
            .unwrap();
        seed_uref
    }

    pub fn get_account_purse_uref(&self, account: AccountHash) -> URef {
        self.builder.get_expected_account(account).main_purse()
    }

    #[allow(dead_code)]
    pub fn get_contract_purse_balance(&self, account: AccountHash) -> U512 {
        let contract_purse_uref: URef = *self
            .get_contract_named_key("kairos_demo_contract", "kairos_deposit_purse", account)
            .as_uref()
            .unwrap();
        self.builder.get_purse_balance(contract_purse_uref)
    }

    /*// read a u64 counter from the contract named keys e.g. "last_processed_deposit_counter"
    pub fn read_counter_value(&mut self, account: AccountHash, name: &str) -> u64 {
        let contract_hash = self.contract_hash("kairos_demo_contract", account);
        self.builder.get_value(contract_hash, name)
    }*/

    // see deposit-session
    pub fn run_deposit_session(&mut self, amount: U512, installer: AccountHash, user: AccountHash) {
        let session_args = runtime_args! {
            "amount" => amount,
            "demo_contract" => self.contract_hash("kairos_demo_contract", installer)
        };
        let session_request = ExecuteRequestBuilder::standard(
            user,
            &format!(
                "{}/{}",
                *PATH_TO_WASM_BINARIES, "deposit-session-optimized.wasm"
            ),
            session_args,
        )
        .build();
        self.builder.exec(session_request).expect_success().commit();
    }

    // see malicious-session
    pub fn run_malicious_session(
        &mut self,
        msg_sender: AccountHash,
        amount: U512,
        account: AccountHash,
    ) {
        let session_args = runtime_args! {
            "amount" => amount,
            "demo_contract" => self.contract_hash("kairos_demo_contract", account)
        };
        let session_request = ExecuteRequestBuilder::standard(
            msg_sender,
            &format!(
                "{}/{}",
                *PATH_TO_WASM_BINARIES, "malicious-session-optimized.wasm"
            ),
            session_args,
        )
        .build();
        self.builder.exec(session_request).expect_failure().commit();
    }

    // see malicious-reader
    pub fn run_malicious_reader_session(
        &mut self,
        msg_sender: AccountHash,
        amount: U512,
        account: AccountHash,
        deposit_purse_uref: URef,
    ) {
        let session_args = runtime_args! {
            "amount" => amount,
            "demo_contract" => self.contract_hash("kairos_demo_contract", account),
            "purse_uref" => deposit_purse_uref
        };
        let session_request = ExecuteRequestBuilder::standard(
            msg_sender,
            &format!(
                "{}/{}",
                *PATH_TO_WASM_BINARIES, "malicious-reader-optimized.wasm"
            ),
            session_args,
        )
        .build();
        self.builder.exec(session_request).expect_failure().commit();
    }
}
