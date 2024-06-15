#![no_std]
#![no_main]
extern crate alloc;
use alloc::string::ToString;
use alloc::vec;
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_event_standard::Schemas;
use casper_types::bytesrepr::Bytes;
use casper_types::{
    contracts::NamedKeys, runtime_args, AccessRights, ApiError, CLValue, EntryPoints, Key,
    RuntimeArgs, URef, U512,
};
use contract_utils::constants::{
    KAIROS_CONTRACT_HASH, KAIROS_CONTRACT_PACKAGE_HASH, KAIROS_CONTRACT_UREF, KAIROS_DEPOSIT_PURSE,
    KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER, KAIROS_TRIE_ROOT, RUNTIME_ARG_AMOUNT,
    RUNTIME_ARG_INITIAL_TRIE_ROOT, RUNTIME_ARG_RECEIPT, RUNTIME_ARG_TEMP_PURSE,
};
use contract_utils::Deposit;
mod entry_points;
mod utils;
use risc0_zkvm::Receipt;
use utils::errors::DepositError;
use utils::get_immediate_caller;

use kairos_circuit_logic::ProofOutputs;

// This entry point is called once when the contract is installed.
// The contract purse will be created in contract context so that it is "owned" by the contract
// rather than the installing account.
#[no_mangle]
pub extern "C" fn init() {
    if runtime::get_key(KAIROS_DEPOSIT_PURSE).is_some() {
        runtime::revert(DepositError::AlreadyInitialized);
    }

    // initialize event schema
    let schemas = Schemas::new().with::<Deposit>();
    casper_event_standard::init(schemas);

    let new_deposit_purse: URef = system::create_purse();
    runtime::put_key(KAIROS_DEPOSIT_PURSE, new_deposit_purse.into());
}

#[no_mangle]
pub extern "C" fn get_purse() {
    let deposit_purse: URef = runtime::get_key(KAIROS_DEPOSIT_PURSE)
        .unwrap_or_revert_with(DepositError::MissingKeyDepositPurse)
        .into_uref()
        .unwrap_or_revert();
    let reference_to_deposit_purse_with_restricted_access =
        deposit_purse.with_access_rights(AccessRights::ADD);
    runtime::ret(
        CLValue::from_t(reference_to_deposit_purse_with_restricted_access)
            .unwrap_or_revert_with(DepositError::FailedToReturnContractPurseAsReference),
    );
}

// Entry point called by a user through session code to deposit funds.
// Due to Casper < 2.0 purse management and access control, it is necessary that
// a temporary purse is funded and passed to the contract, since this is
// the only secure method of making a payment to a contract purse.
#[no_mangle]
pub extern "C" fn deposit() {
    let temp_purse: URef = runtime::get_named_arg(RUNTIME_ARG_TEMP_PURSE);
    let amount: U512 = runtime::get_named_arg(RUNTIME_ARG_AMOUNT);
    let deposit_purse_uref: URef = runtime::get_key(KAIROS_DEPOSIT_PURSE)
        .unwrap_or_revert_with(DepositError::MissingKeyDepositPurse)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);
    system::transfer_from_purse_to_purse(temp_purse, deposit_purse_uref, amount, None)
        .unwrap_or_revert();

    // kairos utilizes u64 so only amounts that can be converted are accepted.
    let amount =
        u64::try_from(amount).unwrap_or_else(|_| runtime::revert(ApiError::InvalidArgument));

    let new_deposit_record: Deposit = Deposit {
        depositor: get_immediate_caller().unwrap_or_revert(),
        amount,
    };
    // this increases a counter automatically - we don't need to create one ourselves
    casper_event_standard::emit(new_deposit_record);
}

#[no_mangle]
pub extern "C" fn submit_batch() {
    let receipt_serialized: Bytes = runtime::get_named_arg(RUNTIME_ARG_RECEIPT);
    let Ok(receipt): Result<Receipt, _> = serde_json_wasm::from_slice(&receipt_serialized) else {
        runtime::revert(ApiError::User(0u16));
    };

    let Ok(ProofOutputs {
        pre_batch_trie_root,
        post_batch_trie_root,
        deposits: _,    // TODO: implement deposits
        withdrawals: _, // TODO: implement withdrawals
    }) = kairos_verifier_risc0_lib::verifier::verify_execution(&receipt)
    else {
        runtime::revert(ApiError::User(1u16));
    };

    // todo: check that the deposits are unique

    // get the current root from contract storage
    let trie_root_uref: URef = runtime::get_key(KAIROS_TRIE_ROOT)
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert_with(ApiError::User(2u16));
    let trie_root: Option<[u8; 32]> = storage::read(trie_root_uref)
        .unwrap_or_revert()
        .unwrap_or_revert_with(ApiError::User(3u16));

    // revert if the previous root of the proof doesn't match the current root
    if trie_root != pre_batch_trie_root {
        runtime::revert(ApiError::User(4u16))
    };
    // store the new root under the contract URef
    storage::write(trie_root_uref, post_batch_trie_root);
    // todo: update sliding window
}

#[no_mangle]
pub extern "C" fn call() {
    let entry_points = EntryPoints::from(vec![
        entry_points::init(),
        entry_points::get_purse(),
        entry_points::deposit(),
        entry_points::submit_batch(),
    ]);

    // this counter will be udpated by the entry point that processes / verifies batches
    let last_processed_deposit_counter_uref: URef = storage::new_uref(0u64);

    let initial_trie_root: Option<[u8; 32]> = runtime::get_named_arg(RUNTIME_ARG_INITIAL_TRIE_ROOT);

    let trie_root_uref: URef = storage::new_uref(initial_trie_root);
    let named_keys = NamedKeys::from([
        (
            KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER.to_string(),
            last_processed_deposit_counter_uref.into(),
        ),
        (KAIROS_TRIE_ROOT.to_string(), trie_root_uref.into()),
    ]);

    let (contract_hash, _) = storage::new_locked_contract(
        entry_points,
        Some(named_keys),
        Some(KAIROS_CONTRACT_PACKAGE_HASH.to_string()),
        Some(KAIROS_CONTRACT_UREF.to_string()),
    );

    let contract_hash_key = Key::from(contract_hash);
    runtime::put_key(KAIROS_CONTRACT_HASH, contract_hash_key);

    // Call the init entry point of the newly installed contract
    // This will setup the deposit purse and initialize Event Schemas (CES)
    let init_args = runtime_args! {};
    runtime::call_contract::<()>(contract_hash, "init", init_args);
}
