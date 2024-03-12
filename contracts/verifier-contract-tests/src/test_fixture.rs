mod utils;
use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    PRODUCTION_RUN_GENESIS_REQUEST,
};
use casper_types::{
    account::AccountHash, runtime_args, Key,
    URef, U512, contracts::NamedKeys, ContractHash, RuntimeArgs
};
use utils::create_funded_dummy_account;
use lazy_static::lazy_static;
extern crate dotenv;
use dotenv::dotenv;
use std::env;
use serde_json;
use host::prove_state_transition;
use kairos_risc0_types::{KairosDeltaTree, hash_bytes, Transfer, Deposit, Withdrawal, TransactionBatch, RiscZeroProof};


pub const ACCOUNT_USER_1: [u8; 32] = [1u8; 32];
pub const ACCOUNT_USER_2: [u8; 32] = [2u8; 32];
pub const ACCOUNT_USER_3: [u8; 32] = [3u8; 32];

// This defines a static variable for the path to WASM binaries
lazy_static! {
    static ref PATH_TO_WASM_BINARIES: String = {
        dotenv().ok(); // Load the .env file at runtime
        env::var("PATH_TO_WASM_BINARIES").expect("Missing environment variable PATH_TO_WASM_BINARIES")
    };
}

#[cfg(test)]
pub struct TestContext {
    pub builder: InMemoryWasmTestBuilder,
    pub account_1: AccountHash,
    pub account_2: AccountHash,
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

    pub fn contract_named_keys(&self, contract_name: &str, key_name: &str, account: AccountHash) -> Key {
        let contract_hash = self.contract_hash_from_named_keys(contract_name, account);
        *self
            .builder
            .get_contract(contract_hash)
            .expect("should have contract")
            .named_keys()
            .get(key_name)
            .unwrap()
    }

    pub fn contract_hash_from_named_keys(&self, key_name: &str, account: AccountHash) -> ContractHash {
        self.named_keys(account)
            .get(key_name)
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash")
    }
    
    pub fn contract_hash(&self, name: &str, account: AccountHash) -> ContractHash{
        self.builder.get_expected_account(account)
            .named_keys()
            .get(name)
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash")
    }

    pub fn install(&mut self, admin: AccountHash) {
        let session_args = runtime_args! {};
        let install_contract_request =
            ExecuteRequestBuilder::standard(
                admin, 
                &format!("{}/{}", *PATH_TO_WASM_BINARIES, "verifier-contract-optimized.wasm"), session_args)
                .build();
        self.builder
            .exec(install_contract_request)
            .expect_success()
            .commit();
    }

    pub fn submit_batch(&mut self, account: AccountHash){
        // prepare a proof
        let mut tree: KairosDeltaTree = KairosDeltaTree{
            zero_node: hash_bytes(vec![0;32]),
            zero_levels: Vec::new(),
            filled: vec![vec![], vec![], vec![], vec![], vec![]],
            root: None,
            index: 0,
            depth: 5
        };
        tree.calculate_zero_levels();
        let transfers: Vec<Transfer> = vec![];
        let deposits: Vec<Deposit> = vec![];
        let withdrawals: Vec<Withdrawal> = vec![];
        let batch: TransactionBatch = TransactionBatch{
            transfers,
            deposits, 
            withdrawals
        };
        let proof: RiscZeroProof = prove_state_transition(tree, batch);

        /*let contract_hash = self.contract_hash("kairos_verifier_contract", account);
        let session_args = runtime_args! {
            "proof" => serde_json::to_vec(&proof).expect("Failed to serialize proof!")
        };*/
        /*
        let create_contract_purse_request = ExecuteRequestBuilder::contract_call_by_hash(
            account,
            contract_hash,
            "submit_batch",
            session_args,
        )
        .build();
        self.builder
            .exec(create_contract_purse_request)
            .expect_success()
            .commit();
        */
    }
}
