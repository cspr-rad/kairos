use core::convert::TryInto;
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};

use casper_types::{bytesrepr::FromBytes, system::CallStackElement, ApiError, CLTyped, URef, Key};
use crate::{error::Error};

fn call_stack_element_to_address(call_stack_element: CallStackElement) -> Key {
    match call_stack_element {
        CallStackElement::Session { account_hash } => Key::from(account_hash),
        CallStackElement::StoredSession { account_hash, .. } => {
            // Stored session code acts in account's context, so if stored session wants to interact
            // with an CEP-18 token caller's address will be used.
            Key::from(account_hash)
        }
        CallStackElement::StoredContract {
            contract_package_hash,
            ..
        } => Key::from(contract_package_hash),
    }
}

/// Gets the immediate session caller of the current execution.
///
/// This function ensures that only session code can execute this function, and disallows stored
/// session/stored contracts.
pub(crate) fn get_immediate_caller() -> Result<Key, Error> {
    let call_stack = runtime::get_call_stack();
    call_stack
        .into_iter()
        .rev()
        .nth(1)
        .map(call_stack_element_to_address)
        .ok_or(Error::InvalidContext)
}