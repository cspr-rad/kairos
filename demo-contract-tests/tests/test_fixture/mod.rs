mod wasm_helper;

use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, WasmTestBuilder, ARG_AMOUNT, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use casper_execution_engine::{
    core::{engine_state, execution},
    storage::global_state::in_memory::InMemoryGlobalState,
};
use casper_types::{
    account::AccountHash,
    bytesrepr::Bytes,
    crypto::{PublicKey, SecretKey},
    runtime_args,
    system::{handle_payment::ARG_TARGET, mint::ARG_ID},
    ApiError, RuntimeArgs, U512,
};
use rand::Rng;
use std::path::Path;

use casper_engine_test_support::{InMemoryWasmTestBuilder, PRODUCTION_RUN_GENESIS_REQUEST};
use casper_types::{ContractHash, URef};

use self::wasm_helper::get_wasm_directory;

pub const ADMIN_SECRET_KEY: [u8; 32] = [1u8; 32];

#[derive(Default)]
pub struct TestContext {
    builder: InMemoryWasmTestBuilder,
    pub admin: AccountHash,
    contract_hash: ContractHash,
    contract_purse: URef,
}

impl TestContext {
    pub fn new(initial_trie_root: Option<[u8; 32]>) -> TestContext {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST);

        let admin_secret_key = SecretKey::ed25519_from_bytes(ADMIN_SECRET_KEY).unwrap();
        let admin = create_funded_account_for_secret_key_bytes(&mut builder, admin_secret_key)
            .to_account_hash();
        let contract_path = get_wasm_directory().0.join("demo-contract-optimized.wasm");
        run_session_with_args(
            &mut builder,
            &contract_path,
            admin,
            runtime_args! {"initial_trie_root" => initial_trie_root },
        );

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

    pub fn create_funded_user(&mut self) -> PublicKey {
        let mut random_secret_key: [u8; 32] = rand::random();
        while random_secret_key == ADMIN_SECRET_KEY {
            random_secret_key = rand::random();
        }

        create_funded_account_for_secret_key_bytes(
            &mut self.builder,
            SecretKey::ed25519_from_bytes(random_secret_key).unwrap(),
        )
    }

    pub fn create_funded_account_for_secret_key(&mut self, secret_key: SecretKey) -> PublicKey {
        create_funded_account_for_secret_key_bytes(&mut self.builder, secret_key)
    }

    pub fn get_user_balance(&mut self, user: AccountHash) -> U512 {
        let user_uref = self.builder.get_expected_account(user).main_purse();
        self.builder.get_purse_balance(user_uref)
    }

    pub fn get_contract_balance(&mut self) -> U512 {
        self.builder.get_purse_balance(self.contract_purse)
    }

    pub fn deposit_succeeds(&mut self, depositor: PublicKey, amount: U512) {
        let account_hash = depositor.to_account_hash();

        let deposit_session_path = get_wasm_directory()
            .1
            .join("deposit-session-optimized.wasm");
        let session_args = runtime_args! {
            "amount" => amount,
            "demo_contract" => self.contract_hash,
            "recipient" => depositor,
        };
        run_session_with_args(
            &mut self.builder,
            deposit_session_path.as_path(),
            account_hash,
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
        let malicious_session_path = get_wasm_directory()
            .1
            .join("malicious-session-optimized.wasm");
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
        let malicious_reader_session_path = get_wasm_directory()
            .1
            .join("malicious-reader-optimized.wasm");
        run_session_with_args(
            &mut self.builder,
            malicious_reader_session_path.as_path(),
            receiver,
            session_args,
        );
        self.builder.expect_failure();
    }

    fn submit_proof_to_contract_commit(&mut self, sender: AccountHash, proof_serialized: Vec<u8>) {
        let session_args = runtime_args! {
            "risc0_receipt" => Bytes::from(proof_serialized),
        };
        let payment = U512::from(3_000_000_000_000u64); // 3000 CSPR
        let submit_batch_request = contract_call_by_hash(
            sender,
            self.contract_hash,
            "submit_batch",
            session_args,
            payment,
        )
        .build();
        self.builder.exec(submit_batch_request).commit();
    }

    pub fn submit_proof_to_contract_expect_success(
        &mut self,
        sender: AccountHash,
        proof_serialized: Vec<u8>,
    ) {
        self.submit_proof_to_contract_commit(sender, proof_serialized);
        self.builder.expect_success();
    }

    pub fn submit_proof_to_contract_expect_api_err(
        &mut self,
        sender: AccountHash,
        proof_serialized: Vec<u8>,
    ) -> ApiError {
        self.submit_proof_to_contract_commit(sender, proof_serialized);

        let exec_results = self
            .builder
            .get_last_exec_results()
            .expect("Expected to be called after run()");

        // not sure about first here it's what the upstream code does
        let exec_result = exec_results
            .first()
            .expect("Unable to get first deploy result");

        match exec_result.as_error() {
            Some(engine_state::Error::Exec(execution::Error::Revert(err))) => *err,
            Some(err) => panic!("Expected revert ApiError, got {:?}", err),
            None => panic!("Expected error"),
        }
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
    account_secret_key: SecretKey,
) -> PublicKey {
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
    account_public_key
}

/// Returns an [`ExecuteRequest`] that will call a stored contract by hash.
/// NOTE: This utiliity is a custom version of `ExecuteRequest::contract_call_by_hash`
/// that allows to specify custom payment.
pub fn contract_call_by_hash(
    sender: AccountHash,
    contract_hash: ContractHash,
    entry_point: &str,
    args: RuntimeArgs,
    payment: U512,
) -> ExecuteRequestBuilder {
    let mut rng = rand::thread_rng();
    let deploy_hash = rng.gen();

    let deploy = DeployItemBuilder::new()
        .with_address(sender)
        .with_stored_session_hash(contract_hash, entry_point, args)
        .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => payment, })
        .with_authorization_keys(&[sender])
        .with_deploy_hash(deploy_hash)
        .build();

    ExecuteRequestBuilder::new().push_deploy(deploy)
}
