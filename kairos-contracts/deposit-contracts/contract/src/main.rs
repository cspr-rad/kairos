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
    KAIROS_DEPOSIT_CONTRACT_PACKAGE, KAIROS_DEPOSIT_PURSE, KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER,
    RUNTIME_ARG_AMOUNT, RUNTIME_ARG_TEMP_PURSE, SECURITY_BADGES,
};
mod utils;
use utils::{get_immediate_caller, get_optional_named_arg_with_user_errors};
mod error;
use error::DepositError;
mod security;
use security::{access_control_check, SecurityBadge};
mod entry_points;
mod events;
use casper_event_standard::Schemas;
use events::Deposit;

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
    let security_badges_dict = storage::new_dictionary(SECURITY_BADGES)
        .unwrap_or_revert_with(DepositError::FailedToCreateSecurityBadgesDict);
    let installing_entity = runtime::get_caller();

    // initialize event schema
    let schemas = Schemas::new().with::<Deposit>();
    casper_event_standard::init(schemas);

    // Assign the admin role to the installer, regardless of the list of admins that was
    // passed to the installation session.
    // The installer is by default an admin and the installer's admin role
    // needs to be revoked after the initialization if it is not wanted.
    storage::dictionary_put(
        security_badges_dict,
        &base64::encode(Key::from(installing_entity).to_bytes().unwrap_or_revert()),
        Some(SecurityBadge::Admin),
    );
    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, DepositError::InvalidAdminList);
    if let Some(admin_list) = admin_list {
        for admin in admin_list {
            let account_dictionary_key = admin.to_bytes().unwrap_or_revert();
            storage::dictionary_put(
                security_badges_dict,
                &base64::encode(account_dictionary_key),
                Some(SecurityBadge::Admin),
            );
        }
    };
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
    casper_event_standard::emit(new_deposit_record);
}

// The centralized Kairos service, or a sequencer,
// will update the counter to keep track
// of the last processed deposit index on-chain.
#[no_mangle]
pub extern "C" fn incr_last_processed_deposit_counter() {
    access_control_check(vec![SecurityBadge::Admin]);
    let last_processed_deposit_counter_uref =
        runtime::get_key(KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER)
            .unwrap_or_revert_with(DepositError::MissingKeyLastProcessedDepositCounter)
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

// Update the security badge for one or multiple accounts
// This entry point is used to assign and revoke roles such
// as the "Admin" role.
#[no_mangle]
pub extern "C" fn update_security_badges() {
    access_control_check(vec![SecurityBadge::Admin]);
    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, DepositError::InvalidAdminList);
    // construct a new admin list from runtime arg
    let mut badge_map: BTreeMap<Key, Option<SecurityBadge>> = BTreeMap::new();
    if let Some(admin_list) = admin_list {
        for account_key in admin_list {
            badge_map.insert(account_key, Some(SecurityBadge::Admin));
        }
    }
    // Remove the caller from the admin list,
    // by inserting None as the security badge.
    // Accounts with no security badge will not be considered part of a badge group
    // and therefore loose the ability to call EPs that contain
    // `access_control_check(vec![SecurityBadge::Admin]);` o.e.
    let caller = get_immediate_caller().unwrap_or_revert();
    badge_map.insert(caller, None);
    security::update_security_badges(&badge_map);
}

#[no_mangle]
pub extern "C" fn call() {
    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, DepositError::InvalidAdminList);

    let entry_points = {
        let mut entry_points = EntryPoints::new();
        entry_points.add_entry_point(entry_points::init());
        entry_points.add_entry_point(entry_points::get_purse());
        entry_points.add_entry_point(entry_points::deposit());
        entry_points.add_entry_point(entry_points::incr_last_processed_deposit_counter());
        entry_points.add_entry_point(entry_points::change_security());
        entry_points
    };
    let mut named_keys = NamedKeys::new();
    let last_processed_deposit_counter = storage::new_uref(u64::from(0u8));

    named_keys.insert(
        KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER.to_string(),
        last_processed_deposit_counter.into(),
    );

    let (contract_hash, _) = storage::new_locked_contract(
        entry_points,
        Some(named_keys),
        Some(KAIROS_DEPOSIT_CONTRACT.to_string()),
        Some(KAIROS_DEPOSIT_CONTRACT_PACKAGE.to_string()),
    );
    let contract_hash_key = Key::from(contract_hash);
    runtime::put_key(KAIROS_DEPOSIT_CONTRACT_NAME, contract_hash_key);

    // Prepare runtime arguments for contract initialization,
    // passing the list of admin accounts that was passed
    // to the installation session.
    let mut init_args = runtime_args! {};
    if let Some(admin_list) = admin_list {
        init_args.insert(ADMIN_LIST, admin_list).unwrap_or_revert();
    }
    runtime::call_contract::<()>(contract_hash, "init", init_args);
}
