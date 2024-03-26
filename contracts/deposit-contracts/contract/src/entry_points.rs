use crate::constants::{RUNTIME_ARG_AMOUNT, RUNTIME_ARG_DEST_PURSE, RUNTIME_ARG_TEMP_PURSE};
use alloc::vec;
use casper_types::{CLType, EntryPoint, EntryPointAccess, EntryPointType, Parameter};

pub fn init() -> EntryPoint {
    EntryPoint::new(
        "init",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn get_purse() -> EntryPoint {
    EntryPoint::new(
        "get_purse",
        vec![],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn deposit() -> EntryPoint {
    EntryPoint::new(
        "deposit",
        vec![
            Parameter::new(RUNTIME_ARG_AMOUNT, CLType::U512),
            Parameter::new(RUNTIME_ARG_TEMP_PURSE, CLType::URef),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn withdrawal() -> EntryPoint {
    EntryPoint::new(
        "withdrawal",
        vec![
            Parameter::new(RUNTIME_ARG_DEST_PURSE, CLType::URef),
            Parameter::new(RUNTIME_ARG_AMOUNT, CLType::U512),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn incr_last_processed_deposit_counter() -> EntryPoint {
    EntryPoint::new(
        "incr_last_processed_deposit_counter",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn change_security() -> EntryPoint {
    EntryPoint::new(
        "change_security",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}
