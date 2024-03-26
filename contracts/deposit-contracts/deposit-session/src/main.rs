#![no_std]
#![no_main]
use casper_contract::contract_api::{account, runtime, system};
use casper_types::{runtime_args, ContractHash, RuntimeArgs, URef, U512};

#[no_mangle]
pub extern "C" fn call() {
    let contract_hash: ContractHash = runtime::get_named_arg("deposit_contract");
    let amount: U512 = runtime::get_named_arg("amount");
    let source: URef = account::get_main_purse();
    // create a temporary purse that can be passed to the deposit contract
    let temp_purse: URef = system::create_purse();
    // fund the temporary purse
    system::transfer_from_purse_to_purse(source, temp_purse, amount, None).unwrap();
    // call the deposit endpoint
    runtime::call_contract::<()>(
        contract_hash,
        "deposit",
        runtime_args! {
            "temp_purse" => temp_purse,
            "amount" => amount
        },
    );
}
