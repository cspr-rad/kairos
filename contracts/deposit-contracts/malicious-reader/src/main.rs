/*
    This session code emulates an attack where a user tries to transfer funds
    out of the deposit contract's purse, by passing the deposit contract purse as a runtime argument
    and calling transfer_from_purse_to_purse
*/

#![no_std]
#![no_main]
use casper_contract::contract_api::{account, runtime, system};
use casper_types::{RuntimeArgs, URef, U512};

#[no_mangle]
pub extern "C" fn call() {
    let amount: U512 = runtime::get_named_arg("amount");
    let purse_uref: URef = runtime::get_named_arg("purse_uref");
    let destination_purse: URef = account::get_main_purse();
    system::transfer_from_purse_to_purse(purse_uref, destination_purse, amount, None).unwrap();
}
