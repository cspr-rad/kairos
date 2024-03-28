use crate::constants::{
    EP_DEPOSIT_NAME, EP_GET_PURSE_NAME, EP_INCR_LAST_PROCESSED_NAME, EP_INIT_NAME,
    EP_UPDATE_SECURITY_BADGES_NAME, RUNTIME_ARG_AMOUNT, RUNTIME_ARG_DEST_PURSE,
    RUNTIME_ARG_TEMP_PURSE,
};
use alloc::vec;
use casper_types::{CLType, EntryPoint, EntryPointAccess, EntryPointType, Parameter};

pub fn init() -> EntryPoint {
    EntryPoint::new(
        EP_INIT_NAME,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn get_purse() -> EntryPoint {
    EntryPoint::new(
        EP_GET_PURSE_NAME,
        vec![],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn deposit() -> EntryPoint {
    EntryPoint::new(
        EP_DEPOSIT_NAME,
        vec![
            Parameter::new(RUNTIME_ARG_AMOUNT, CLType::U512),
            Parameter::new(RUNTIME_ARG_TEMP_PURSE, CLType::URef),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn incr_last_processed_deposit_counter() -> EntryPoint {
    EntryPoint::new(
        EP_INCR_LAST_PROCESSED_NAME,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn change_security() -> EntryPoint {
    EntryPoint::new(
        EP_UPDATE_SECURITY_BADGES_NAME,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}
