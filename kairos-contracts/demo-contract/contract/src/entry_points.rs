use crate::constants::{
    EP_DEPOSIT_NAME, EP_GET_PURSE_NAME, EP_INIT_NAME, EP_SUBMIT_BATCH_NAME, RUNTIME_ARG_AMOUNT,
    RUNTIME_ARG_BATCH, RUNTIME_ARG_TEMP_PURSE, RUNTIME_ARG_TX,
};
use alloc::boxed::Box;
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
            Parameter::new(RUNTIME_ARG_TX, CLType::List(Box::new(CLType::U8))),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn submit_batch() -> EntryPoint {
    EntryPoint::new(
        EP_SUBMIT_BATCH_NAME,
        vec![Parameter::new(RUNTIME_ARG_BATCH, CLType::Any)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}
