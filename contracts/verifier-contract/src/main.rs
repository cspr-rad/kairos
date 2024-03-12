#![no_std]
#![no_main]
extern crate alloc;
use alloc::{string::ToString, vec, vec::Vec};
use casper_contract::{
    contract_api::{risc0::{self, risc0_verifier}, runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
mod constants;
use constants::{KAIROS_VERIFIER_CONTRACT, KAIROS_VERIFIER_CONTRACT_NAME, KAIROS_VERIFIER_CONTRACT_PACKAGE};
use casper_types::{
    EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, contracts::NamedKeys,
    AccessRights, ApiError, CLType, CLValue, Key, Parameter, URef, U512, ContractHash
};

#[no_mangle]
pub extern "C" fn submit_batch(){
    let proof: Vec<u8> = runtime::get_named_arg("proof");
    // deserialize and perform checks on journal
    assert_eq!(risc0_verifier(proof), [1u8]);
    // store new tree from deserialized journal and increase index
}

#[no_mangle]
pub extern "C" fn call(){
    let entry_points = {
        let mut entry_points = EntryPoints::new();
        let submit_batch = EntryPoint::new(
            "submit_batch",
            // placeholder
            vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );
        entry_points.add_entry_point(submit_batch);
        entry_points
    };
    let mut named_keys = NamedKeys::new();
    /*
        Todo: create dict to store tree history
        Todo: create counter to track tree index
    */
    let (contract_hash, _) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(KAIROS_VERIFIER_CONTRACT.to_string()),
        Some(KAIROS_VERIFIER_CONTRACT_PACKAGE.to_string()),
    );
    let contract_hash_key = Key::from(contract_hash);
    runtime::put_key(KAIROS_VERIFIER_CONTRACT_NAME, contract_hash_key);
}