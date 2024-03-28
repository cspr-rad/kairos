/*
    The utilities found in this file were scraped from other Casper contracts,
    mainly cep-78 and cep-18.
    This file is not necessarily due for review, unless breaking changes are suspected.
*/

use crate::error::DepositError;
use alloc::vec::Vec;
use casper_contract::{
    contract_api::{self, runtime},
    ext_ffi,
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    api_error, bytesrepr, bytesrepr::FromBytes, system::CallStackElement, ApiError, Key, URef,
};

/// Wrap the immediate caller as a Key and return it
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

/// Traverse the callstack to retrieve the n - 1 th element of the callstack
pub(crate) fn get_immediate_caller() -> Result<Key, DepositError> {
    let call_stack = runtime::get_call_stack();
    call_stack
        .into_iter()
        .rev()
        .nth(1)
        .map(call_stack_element_to_address)
        .ok_or(DepositError::InvalidContext)
}

/// Gets [`URef`] under a name.
pub(crate) fn get_uref(name: &str) -> URef {
    let key = runtime::get_key(name)
        .ok_or(ApiError::MissingKey)
        .unwrap_or_revert();
    key.try_into().unwrap_or_revert()
}

pub fn get_named_arg_size(name: &str) -> Option<usize> {
    let mut arg_size: usize = 0;
    let ret = unsafe {
        ext_ffi::casper_get_named_arg_size(
            name.as_bytes().as_ptr(),
            name.len(),
            &mut arg_size as *mut usize,
        )
    };
    match api_error::result_from(ret) {
        Ok(_) => Some(arg_size),
        Err(ApiError::MissingArgument) => None,
        Err(e) => runtime::revert(e),
    }
}

/// Optional named args are used for access control
pub fn get_optional_named_arg_with_user_errors<T: FromBytes>(
    name: &str,
    invalid: DepositError,
) -> Option<T> {
    let maybe_named_arg_with_user_errors = get_named_arg_with_user_errors::<T>(name, invalid);
    match maybe_named_arg_with_user_errors {
        Ok(val) => Some(val),
        Err(_) => None,
    }
}

/// Reading optional args is required because it should be possible to
/// install the deposit contract without passing a list of
/// admins. In such a case the default admin will be the installing
/// account.
///
/// This function checks if the runtime argument size is greater than 0
/// and if that is the case it gets parsed and returned. Should the parsing
/// fail or should the user not supply the argument at all, then an error is propagated.
///
/// To install the contract without an admin list, one still needs to pass an empty admins
/// list as a runtime argument. Otherwise the missing error will be propagated.
pub fn get_named_arg_with_user_errors<T: FromBytes>(
    name: &str,
    invalid: DepositError,
) -> Result<T, DepositError> {
    let arg_size = get_named_arg_size(name).ok_or(DepositError::MissingOptionalArgument)?;
    let arg_bytes = if arg_size > 0 {
        let res = {
            let data_non_null_ptr = contract_api::alloc_bytes(arg_size);
            let ret = unsafe {
                ext_ffi::casper_get_named_arg(
                    name.as_bytes().as_ptr(),
                    name.len(),
                    data_non_null_ptr.as_ptr(),
                    arg_size,
                )
            };
            let data =
                unsafe { Vec::from_raw_parts(data_non_null_ptr.as_ptr(), arg_size, arg_size) };
            api_error::result_from(ret).map(|_| data)
        };
        // Assumed to be safe as `get_named_arg_size` checks the argument already
        res.unwrap_or_revert_with(DepositError::FailedToGetArgBytes)
    } else {
        // Avoids allocation with 0 bytes and a call to get_named_arg
        Vec::new()
    };
    bytesrepr::deserialize(arg_bytes).map_err(|_| invalid)
}
