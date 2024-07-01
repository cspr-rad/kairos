use alloc::vec;
use casper_types::{CLType, EntryPoint, EntryPointAccess, EntryPointType, Parameter};
use contract_utils::constants::{
    EP_DEPOSIT_NAME, EP_GET_PURSE_NAME, EP_INIT_NAME, EP_SUBMIT_NAME, RUNTIME_ARG_AMOUNT,
    RUNTIME_ARG_RECEIPT, RUNTIME_ARG_RECIPIENT, RUNTIME_ARG_TEMP_PURSE,
};

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
            Parameter::new(RUNTIME_ARG_RECIPIENT, CLType::PublicKey),
            Parameter::new(RUNTIME_ARG_AMOUNT, CLType::U512),
            Parameter::new(RUNTIME_ARG_TEMP_PURSE, CLType::URef),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn submit_batch() -> EntryPoint {
    EntryPoint::new(
        EP_SUBMIT_NAME,
        vec![Parameter::new(RUNTIME_ARG_RECEIPT, CLType::Any)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}
