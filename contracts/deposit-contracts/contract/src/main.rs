#![no_std]
#![no_main]
extern crate alloc;
use alloc::{collections::BTreeMap, string::ToString, vec, vec::Vec};
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr::ToBytes, contracts::NamedKeys, runtime_args, AccessRights, ApiError, CLValue,
    EntryPoints, Key, RuntimeArgs, URef, U512,
};
mod constants;
use constants::{
    ADMIN_LIST, KAIROS_DEPOSIT_CONTRACT, KAIROS_DEPOSIT_CONTRACT_NAME,
    KAIROS_DEPOSIT_CONTRACT_PACKAGE, KAIROS_DEPOSIT_EVENT_DICT, KAIROS_DEPOSIT_PURSE,
    KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER, KAIROS_MOST_RECENT_DEPOSIT_COUNTER, RUNTIME_ARG_AMOUNT,
    RUNTIME_ARG_DEST_PURSE, RUNTIME_ARG_TEMP_PURSE, SECURITY_BADGES,
};
mod detail;
use detail::{get_immediate_caller, get_optional_named_arg_with_user_errors};
mod error;
use error::DepositError;
mod security;
use security::{sec_check, SecurityBadge};
mod entry_points;

use contract_types::Deposit;

#[no_mangle]
pub extern "C" fn init() {
    if runtime::get_key(KAIROS_DEPOSIT_PURSE).is_some() {
        runtime::revert(DepositError::AlreadyInitialized);
    }
    let security_badges_dict = storage::new_dictionary(SECURITY_BADGES).unwrap_or_revert();
    storage::dictionary_put(
        security_badges_dict,
        &base64::encode(
            Key::from(runtime::get_caller())
                .to_bytes()
                .unwrap_or_revert(),
        ),
        SecurityBadge::Admin,
    );
    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, DepositError::InvalidAdminList);
    if let Some(admin_list) = admin_list {
        for admin in admin_list {
            storage::dictionary_put(
                security_badges_dict,
                &base64::encode(admin.to_bytes().unwrap_or_revert()),
                SecurityBadge::Admin,
            );
        }
    };
    let new_deposit_purse: URef = system::create_purse();
    runtime::put_key(KAIROS_DEPOSIT_PURSE, new_deposit_purse.into());
}

#[no_mangle]
pub extern "C" fn get_purse() {
    let deposit_purse: URef = runtime::get_key(KAIROS_DEPOSIT_PURSE)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert();
    runtime::ret(
        CLValue::from_t(deposit_purse.with_access_rights(AccessRights::ADD)).unwrap_or_revert(),
    );
}

#[no_mangle]
pub extern "C" fn deposit() {
    let temp_purse: URef = runtime::get_named_arg(RUNTIME_ARG_TEMP_PURSE);
    let amount: U512 = runtime::get_named_arg(RUNTIME_ARG_AMOUNT);
    let deposit_purse_uref: URef = runtime::get_key(KAIROS_DEPOSIT_PURSE)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);
    system::transfer_from_purse_to_purse(temp_purse, deposit_purse_uref, amount, None)
        .unwrap_or_revert();

    let most_recent_deposit_counter_uref = runtime::get_key(KAIROS_MOST_RECENT_DEPOSIT_COUNTER)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert();
    let mut most_recent_deposit_counter_value: u64 =
        storage::read(most_recent_deposit_counter_uref)
            .unwrap_or_revert()
            .unwrap_or_revert();
    let new_deposit_record: Deposit = Deposit {
        account: get_immediate_caller().unwrap_or_revert(),
        amount,
        timestamp: None,
        processed: false,
    };

    let kairos_deposit_event_dict_uref = runtime::get_key(KAIROS_DEPOSIT_EVENT_DICT)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert();
    storage::dictionary_put::<Vec<u8>>(
        kairos_deposit_event_dict_uref,
        &most_recent_deposit_counter_value.to_string(),
        bincode::serialize(&new_deposit_record).unwrap(),
    );

    most_recent_deposit_counter_value += 1u64;
    storage::write(
        most_recent_deposit_counter_uref,
        most_recent_deposit_counter_value,
    );
}

#[no_mangle]
pub extern "C" fn incr_last_processed_deposit_counter() {
    sec_check(vec![SecurityBadge::Admin]);
    let last_processed_deposit_counter_uref =
        runtime::get_key(KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER)
            .unwrap_or_revert_with(ApiError::MissingKey)
            .into_uref()
            .unwrap_or_revert();
    let mut last_processed_deposit_counter_value: u64 =
        storage::read(last_processed_deposit_counter_uref)
            .unwrap_or_revert()
            .unwrap_or_revert();
    last_processed_deposit_counter_value += 1u64;
    storage::write(
        last_processed_deposit_counter_uref,
        last_processed_deposit_counter_value,
    );
}

#[no_mangle]
pub extern "C" fn withdrawal() {
    sec_check(vec![SecurityBadge::Admin]);
    let destination_purse: URef = runtime::get_named_arg(RUNTIME_ARG_DEST_PURSE);
    let amount: U512 = runtime::get_named_arg(RUNTIME_ARG_AMOUNT);
    let deposit_purse: URef = runtime::get_key(KAIROS_DEPOSIT_PURSE)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert()
        .with_access_rights(AccessRights::READ_ADD_WRITE);
    system::transfer_from_purse_to_purse(deposit_purse, destination_purse, amount, None)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn change_security() {
    sec_check(vec![SecurityBadge::Admin]);
    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, DepositError::InvalidAdminList);
    // construct a new admin list from runtime arg
    let mut badge_map: BTreeMap<Key, SecurityBadge> = BTreeMap::new();
    if let Some(admin_list) = admin_list {
        for account_key in admin_list {
            badge_map.insert(account_key, SecurityBadge::Admin);
        }
    }
    // remove the caller from the admin list
    let caller = get_immediate_caller().unwrap_or_revert();
    badge_map.remove(&caller);
    security::change_sec_badge(&badge_map);
}

#[no_mangle]
pub extern "C" fn call() {
    let caller = Key::from(runtime::get_caller());

    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, DepositError::InvalidAdminList);

    let entry_points = {
        let mut entry_points = EntryPoints::new();
        entry_points.add_entry_point(entry_points::init());
        entry_points.add_entry_point(entry_points::get_purse());
        entry_points.add_entry_point(entry_points::deposit());
        entry_points.add_entry_point(entry_points::withdrawal());
        entry_points.add_entry_point(entry_points::incr_last_processed_deposit_counter());
        entry_points.add_entry_point(entry_points::change_security());
        entry_points
    };
    let mut named_keys = NamedKeys::new();
    let event_dict = storage::new_dictionary(KAIROS_DEPOSIT_EVENT_DICT).unwrap_or_revert();
    named_keys.insert(KAIROS_DEPOSIT_EVENT_DICT.to_string(), event_dict.into());
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

    // runtime arguments for contract initialization
    let mut init_args = runtime_args! {};
    if let Some(mut admin_list) = admin_list {
        // for testing: add the caller to the admin list
        admin_list.push(caller);
        // prepare runtime arguments
        init_args.insert(ADMIN_LIST, admin_list).unwrap_or_revert();
    }
    runtime::call_contract::<()>(contract_hash, "init", init_args);
}
