#![no_std]
#![no_main]
extern crate alloc;
use alloc::string::String;
use casper_contract::{
    contract_api::{account, runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{runtime_args, AddressableEntityHash, ApiError, Key, RuntimeArgs, URef, U512};

#[no_mangle]
pub extern "C" fn call() {
    let contract_hash: AddressableEntityHash = runtime::get_named_arg("deposit_contract");
    let amount: U512 = runtime::get_named_arg("amount");
    let purse_uref: URef = runtime::get_named_arg("purse_uref");
    let destination_purse: URef = account::get_main_purse();
    system::transfer_from_purse_to_purse(purse_uref, destination_purse, amount, None).unwrap();
}
