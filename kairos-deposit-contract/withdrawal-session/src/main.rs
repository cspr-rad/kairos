#![no_std]
#![no_main]
use casper_contract::contract_api::{account, runtime};
use casper_types::{runtime_args, AddressableEntityHash, URef, U512};

#[no_mangle]
pub extern "C" fn call() {
    let contract_hash: AddressableEntityHash = runtime::get_named_arg("deposit_contract");
    let amount: U512 = runtime::get_named_arg("amount");
    let destination_purse: URef = account::get_main_purse();
    runtime::call_contract::<()>(
        contract_hash,
        "withdrawal",
        runtime_args! {
            "destination_purse" => destination_purse,
            "amount" => amount
        },
    );
}
