use casper_engine_test_support::{
    ExecuteRequestBuilder, WasmTestBuilder, ARG_AMOUNT, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{
    account::AccountHash,
    runtime_args,
    system::{handle_payment::ARG_TARGET, mint::ARG_ID},
    PublicKey, RuntimeArgs, SecretKey,
};

// Creates a dummy account and transfer funds to it
pub fn create_funded_dummy_account(
    builder: &mut WasmTestBuilder<InMemoryGlobalState>,
    account_string: Option<[u8; 32]>,
) -> AccountHash {
    let (_, account_public_key) =
        create_dummy_key_pair(if let Some(account_string) = account_string {
            account_string
        } else {
            [7u8; 32]
        });
    let account = account_public_key.to_account_hash();
    fund_account(builder, account);
    account
}

pub fn create_dummy_key_pair(account_string: [u8; 32]) -> (SecretKey, PublicKey) {
    let secret_key =
        SecretKey::ed25519_from_bytes(account_string).expect("failed to create secret key");
    let public_key = PublicKey::from(&secret_key);
    (secret_key, public_key)
}

pub fn fund_account(builder: &mut WasmTestBuilder<InMemoryGlobalState>, account: AccountHash) {
    let transfer = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            ARG_AMOUNT => DEFAULT_ACCOUNT_INITIAL_BALANCE / 10_u64,
            ARG_TARGET => account,
            ARG_ID => Option::<u64>::None,
        },
    )
    .build();
    builder.exec(transfer).expect_success().commit();
}
