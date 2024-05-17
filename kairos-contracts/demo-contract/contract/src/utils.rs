// Utilities copied from cep-78 and cep-18 implementation.

use casper_contract::contract_api::runtime;
use casper_types::{system::CallStackElement, Key};

pub mod errors;
pub mod events;
use errors::DepositError;

/// Wrap the immediate caller as a Key and return it
fn call_stack_element_to_key(call_stack_element: CallStackElement) -> Key {
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

/// Traverse the callstack to retrieve the n - 1 th element of the callstack
pub(crate) fn get_immediate_caller() -> Result<Key, DepositError> {
    let call_stack = runtime::get_call_stack();
    call_stack
        .into_iter()
        .rev()
        .nth(1)
        .map(call_stack_element_to_key)
        .ok_or(DepositError::InvalidContext)
}
