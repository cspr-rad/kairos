mod utils;
use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    PRODUCTION_RUN_GENESIS_REQUEST
};
use casper_types::{
    account::AccountHash, runtime_args, Key,
    URef, U512, contracts::NamedKeys, ContractHash, RuntimeArgs, CLValue, CLType, bytesrepr::ToBytes, bytesrepr::FromBytes, bytesrepr::Bytes, bytesrepr
};
use utils::create_funded_dummy_account;
use lazy_static::lazy_static;
extern crate dotenv;
use dotenv::dotenv;
use std::env;
use serde_json;
use kairos_risc0_types::{KairosDeltaTree, hash_bytes, Transfer, Deposit, Withdrawal, TransactionBatch, RiscZeroProof, CircuitArgs, CircuitJournal};
use methods::{
    NATIVE_CSPR_TX_ELF, NATIVE_CSPR_TX_ID
};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use bincode;

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
//pub const PATH_TO_WASM_BINARIES: &str = "/Users/chef/Desktop/kairos-lab/contracts/verifier-contract/target/wasm32-unknown-unknown/release";

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
        /*tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
            .init();*/
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
        let proof: RiscZeroProof = prove_batch(tree, batch);
        let receipt: Receipt = bincode::deserialize(&proof.receipt_serialized).expect("Failed to deserialize receipt!");
        let journal: &CircuitJournal = &receipt.journal.decode::<CircuitJournal>().unwrap();
        let bincode_serialized_proof: Vec<u8> = bincode::serialize(&proof).expect("Failed to serialize proof!");
        let contract_hash = self.contract_hash("kairos_verifier_contract", account);
        println!("Bincode Proof size: {:?} should be less than 1_500_000", &bincode_serialized_proof.len());
        let mut cl_proof = Bytes::from(bincode_serialized_proof);
        let deserialized_proof: RiscZeroProof = bincode::deserialize(&cl_proof.as_slice().as_ref()).unwrap();
        let session_args = runtime_args! {
            "proof" => cl_proof
        };
        let submit_batch_request = ExecuteRequestBuilder::contract_call_by_hash(
            account,
            contract_hash,
            "submit_delta_tree_batch",
            session_args.clone(),
        )
        .build();
        self.builder
            .exec(submit_batch_request)
            .expect_success()
            .commit();
    }
}

pub fn prove_batch(tree: KairosDeltaTree, batch: TransactionBatch) -> RiscZeroProof{
    let inputs = CircuitArgs{
        tree,
        batch
    };
    let env = ExecutorEnv::builder()
    .write(&inputs)
    .unwrap()
    .build()
    .unwrap();

    let prover = default_prover();
    let receipt = prover.prove(env, NATIVE_CSPR_TX_ELF).unwrap();
    receipt.verify(NATIVE_CSPR_TX_ID).expect("Failed to verify proof!");
    RiscZeroProof{
        receipt_serialized: bincode::serialize(&receipt).unwrap(),
        program_id: NATIVE_CSPR_TX_ID.to_vec()
    }
}