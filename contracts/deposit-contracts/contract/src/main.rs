// cfg
#![no_std]
#![no_main]
extern crate alloc;
use alloc::{string::ToString, vec, vec::Vec};
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, contracts::NamedKeys,
    AccessRights, ApiError, CLType, CLValue, Key, Parameter, URef, U512, ContractHash
};
mod constants;
mod detail;
mod error;
use constants::{
    KAIROS_ADMIN, KAIROS_AMOUNT, KAIROS_DEPOSIT_CONTRACT, KAIROS_DEPOSIT_CONTRACT_NAME,
    KAIROS_DEPOSIT_CONTRACT_PACKAGE, KAIROS_DEPOSIT_EVENT_DICT, KAIROS_DEPOSIT_PURSE,
    KAIROS_DESTINATION_PURSE, KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER,
    KAIROS_MOST_RECENT_DEPOSIT_COUNTER, KAIROS_NEW_ADMIN, KAIROS_TEMP_PURSE,
};
use kairos_risc0_types::Deposit;
use detail::get_immediate_caller;

#[no_mangle]
pub extern "C" fn update_admin() {
    let caller = get_immediate_caller().unwrap_or_revert();
    let admin = runtime::get_key(KAIROS_ADMIN).unwrap();
    assert_eq!(caller, admin);

    let new_admin: Key = runtime::get_named_arg(KAIROS_NEW_ADMIN);
    runtime::put_key(KAIROS_ADMIN, new_admin);
}

#[no_mangle]
pub extern "C" fn create_purse() {
    /* ACCESS CONTROLED | ADMIN ONLY
        create a new empty purse and update the value in named_keys
        kairos_deposit_purse: URef
    */
    let caller: Key = get_immediate_caller().unwrap_or_revert();
    let admin = runtime::get_key(KAIROS_ADMIN).unwrap();
    assert_eq!(caller, admin);

    // ! This could be dangerous
    let new_deposit_purse: URef = system::create_purse();
    // we store a purse that can only be added to
    runtime::put_key(KAIROS_DEPOSIT_PURSE, new_deposit_purse.into());
}

#[no_mangle]
pub extern "C" fn get_purse() {
    let deposit_purse: URef = runtime::get_key(KAIROS_DEPOSIT_PURSE)
        .unwrap()
        .into_uref()
        .unwrap();
    runtime::ret(
        CLValue::from_t(deposit_purse.with_access_rights(AccessRights::ADD)).unwrap_or_revert(),
    );
}

#[no_mangle]
pub extern "C" fn deposit() {
    /*
        transfer from source purse to contract purse
    */
    let temp_purse: URef = runtime::get_named_arg(KAIROS_TEMP_PURSE);
    let amount: U512 = runtime::get_named_arg(KAIROS_AMOUNT);
    let deposit_purse_uref: URef = runtime::get_key(KAIROS_DEPOSIT_PURSE)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);
    system::transfer_from_purse_to_purse(temp_purse, deposit_purse_uref, amount, None).unwrap();
    /* get last processed key
        store the serialized deposit struct under the new most_recent_deposit_counter
        most_recent_deposit_counter ++
    */
    let most_recent_deposit_counter_uref = runtime::get_key(KAIROS_MOST_RECENT_DEPOSIT_COUNTER)
        .unwrap()
        .into_uref()
        .unwrap();
    let mut most_recent_deposit_counter_value: u64 =
        storage::read(most_recent_deposit_counter_uref)
            .unwrap()
            .unwrap();
    let new_deposit_record: Deposit = Deposit {
        account: get_immediate_caller().unwrap_or_revert(),
        amount,
        timestamp: None,
        processed: false
    };

    let kairos_deposit_event_dict_uref = runtime::get_key(KAIROS_DEPOSIT_EVENT_DICT)
        .unwrap()
        .into_uref()
        .unwrap();
    storage::dictionary_put::<Vec<u8>>(
        kairos_deposit_event_dict_uref,
        &most_recent_deposit_counter_value.to_string(),
        serde_json_wasm::to_vec(&new_deposit_record).unwrap(),
    );
    most_recent_deposit_counter_value += 1u64;
    storage::write(
        most_recent_deposit_counter_uref,
        most_recent_deposit_counter_value,
    );
}

#[no_mangle]
pub extern "C" fn incr_last_processed_deposit_counter() {
    /* Update the most recent deposit counter
        This EP can only be called by Kairos L2 when a new deposit is recorded.
        The counter will be automatically updated when the deposit EP is called directly on the L1.
    */
    let caller = get_immediate_caller().unwrap_or_revert();
    let admin = runtime::get_key(KAIROS_ADMIN).unwrap();
    assert_eq!(caller, admin);

    let last_processed_deposit_counter_uref =
        runtime::get_key(KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER)
            .unwrap()
            .into_uref()
            .unwrap();
    let mut last_processed_deposit_counter_value: u64 =
        storage::read(last_processed_deposit_counter_uref)
            .unwrap()
            .unwrap();
    last_processed_deposit_counter_value += 1u64;
    storage::write(
        last_processed_deposit_counter_uref,
        last_processed_deposit_counter_value,
    );
}

/*
    Withdrawal from deposit contract (Admin Only)
        -> send funds from contract to user
        -> centralized approach
*/
#[no_mangle]
pub extern "C" fn withdrawal() {
    let caller = get_immediate_caller().unwrap_or_revert();
    let admin = runtime::get_key(KAIROS_ADMIN).unwrap();
    assert_eq!(caller, admin);

    let destination_purse: URef = runtime::get_named_arg(KAIROS_DESTINATION_PURSE);
    let amount: U512 = runtime::get_named_arg(KAIROS_AMOUNT);
    let deposit_purse: URef = runtime::get_key(KAIROS_DEPOSIT_PURSE)
        .unwrap()
        .into_uref()
        .unwrap()
        .with_access_rights(AccessRights::READ_ADD_WRITE);
    system::transfer_from_purse_to_purse(deposit_purse, destination_purse, amount, None).unwrap();
}

#[no_mangle]
pub extern "C" fn call() {
    /*
        Mandatory named_keys:
            * Admin: Key
            * kairos_deposit_purse: URef
    */
    let caller = Key::from(runtime::get_caller()); //get_immediate_caller().unwrap_or_revert();

    let entry_points = {
        let mut entry_points = EntryPoints::new();
        let update_admin = EntryPoint::new(
            "update_admin",
            vec![Parameter::new(KAIROS_NEW_ADMIN, CLType::Key)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );
        let create_purse = EntryPoint::new(
            "create_purse",
            vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );
        let get_purse = EntryPoint::new(
            "get_purse",
            vec![],
            CLType::URef,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );
        let deposit = EntryPoint::new(
            "deposit",
            vec![
                Parameter::new(KAIROS_AMOUNT, CLType::U512),
                Parameter::new(KAIROS_TEMP_PURSE, CLType::URef),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );
        let withdrawal = EntryPoint::new(
            "withdrawal",
            vec![
                Parameter::new(KAIROS_DESTINATION_PURSE, CLType::URef),
                Parameter::new(KAIROS_AMOUNT, CLType::U512),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );
        let incr_last_processed_deposit_counter = EntryPoint::new(
            "incr_last_processed_deposit_counter",
            vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );
        entry_points.add_entry_point(update_admin);
        entry_points.add_entry_point(create_purse);
        entry_points.add_entry_point(get_purse);
        entry_points.add_entry_point(deposit);
        entry_points.add_entry_point(withdrawal);
        entry_points.add_entry_point(incr_last_processed_deposit_counter);
        entry_points
    };
    let mut named_keys = NamedKeys::new();
    // key to store the deposit purse
    named_keys.insert(KAIROS_DEPOSIT_PURSE.to_string(), URef::default().into());
    // key to store the admin account
    named_keys.insert(KAIROS_ADMIN.to_string(), caller);
    // dictionary to store event history
    let event_dict = storage::new_dictionary(KAIROS_DEPOSIT_EVENT_DICT).unwrap();
    named_keys.insert(KAIROS_DEPOSIT_EVENT_DICT.to_string(), event_dict.into());
    // sliding window set of counters
    let last_processed_deposit_counter = storage::new_uref(u64::from(0u8));
    named_keys.insert(
        KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER.to_string(),
        last_processed_deposit_counter.into(),
    );
    let most_recent_deposit_counter = storage::new_uref(u64::from(0u8));
    named_keys.insert(
        KAIROS_MOST_RECENT_DEPOSIT_COUNTER.to_string(),
        most_recent_deposit_counter.into(),
    );

    let (contract_hash, _) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(KAIROS_DEPOSIT_CONTRACT.to_string()),
        Some(KAIROS_DEPOSIT_CONTRACT_PACKAGE.to_string()),
    );
    let contract_hash_key = Key::from(contract_hash);
    runtime::put_key(KAIROS_DEPOSIT_CONTRACT_NAME, contract_hash_key);
}
