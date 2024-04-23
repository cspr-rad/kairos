#![no_std]
#![no_main]
extern crate alloc;
use alloc::{string::ToString, vec::Vec};
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_event_standard::Schemas;
use casper_types::{
    contracts::NamedKeys, runtime_args, AccessRights, ApiError, CLValue, EntryPoints, Key,
    RuntimeArgs, URef, U512,
};
mod constants;
use constants::{
    KAIROS_DEPOSIT_CONTRACT_NAME, KAIROS_DEPOSIT_CONTRACT_PACKAGE, KAIROS_DEPOSIT_CONTRACT_UREF,
    KAIROS_DEPOSIT_PURSE, KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER, RUNTIME_ARG_AMOUNT,
    RUNTIME_ARG_TEMP_PURSE,
};
mod entry_points;
mod utils;
use utils::errors::DepositError;
use utils::events::Deposit;
use utils::get_immediate_caller;

// This entry point is called once when the contract is installed
// and sets up the security badges with the installer as an admin or the
// optional list of admins.
// The optional list of admins is passed to the installation session as a runtime argument.
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

// Batch submission EP with host verifier: Unimplemented
// This entry point will verify a batch proof and update the trie root
// TODO: Store the trie root under a URef in this deposit contract
// QUESTION: What is the initial root and state of the trie?
#[no_mangle]
pub extern "C" fn submit_batch() {
    // receive some serialized proof
    // the proof will include a public journal with the updated trie root
    // public journal will include all deposits
    let _serialized_proof: Vec<u8> = runtime::get_named_arg("serialized_proof");
    // TODO: VERIFY proof and UPDATE the trie root    
    // TODO: UPDATE last_processed_deposit_counter by `n`, where `n` is the amount of deposits processed in this batch.

    // OPEN QUESTIONS
    // How are L1 deposits verified when proving?
    // Are deposits and transfers private or public inputs to the circuit / guest?
    // Do we want to only store the most recent root, or a history of roots?
    // Option<Root>

    // Merkle proof for each state transition in the Trie
    // If that matches the current root it'll be valid
    // DA must store all transitions and those are verified against the root hash?
    // Triestate is stored in the L2
    // The trie is updated inside the guest

    // Trie verifies Balance against L1 root -> not responsible for proving updates
    // Update Trie inside Circuit
}

// Entry point called by a user through session code to deposit funds.
// Due to Casper < 2.0 purse management and access control, it is necessary that
// a temporary purse is funded and passed to the deposit contract, since this is
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

    let new_deposit_record: Deposit = Deposit {
        account: get_immediate_caller().unwrap_or_revert(),
        amount,
        timestamp: None,
    };
    // this increases a counter automatically - we don't need to create one ourselves
    casper_event_standard::emit(new_deposit_record);
}

#[no_mangle]
pub extern "C" fn call() {
    let entry_points = {
        let mut entry_points = EntryPoints::new();
        entry_points.add_entry_point(entry_points::init());
        entry_points.add_entry_point(entry_points::get_purse());
        entry_points.add_entry_point(entry_points::deposit());
        entry_points
    };
    // this counter will be udpated by the entry point that processes / verifies batches
    let mut named_keys = NamedKeys::new();
    let last_processed_deposit_counter = storage::new_uref(u64::from(0u8));

    named_keys.insert(
        KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER.to_string(),
        last_processed_deposit_counter.into(),
    );

    let (contract_hash, _) = storage::new_locked_contract(
        entry_points,
        Some(named_keys),
        Some(KAIROS_DEPOSIT_CONTRACT_UREF.to_string()),
        Some(KAIROS_DEPOSIT_CONTRACT_PACKAGE.to_string()),
    );
    let contract_hash_key = Key::from(contract_hash);
    runtime::put_key(KAIROS_DEPOSIT_CONTRACT_NAME, contract_hash_key);

    let init_args = runtime_args! {};
    // Call the init entry point of the newly installed contract
    // This will setup the deposit purse and initialize Event Schemas (CES)
    runtime::call_contract::<()>(contract_hash, "init", init_args);
}
