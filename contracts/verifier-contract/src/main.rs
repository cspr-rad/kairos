#![no_std]
#![no_main]
extern crate alloc;
use alloc::{string::ToString, vec, vec::Vec};
use casper_contract::{
    contract_api::{risc0::{self, risc0_verifier}, runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
mod constants;
use constants::{KAIROS_VERIFIER_CONTRACT, KAIROS_VERIFIER_CONTRACT_NAME, KAIROS_VERIFIER_CONTRACT_PACKAGE, RUNTIME_ARG_PROOF};
use casper_types::{
    EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, contracts::NamedKeys,
    AccessRights, ApiError, CLType, CLValue, Key, Parameter, URef, U512, ContractHash, bytesrepr::ToBytes, bytesrepr::FromBytes, bytesrepr::Bytes
};
mod error;
use error::RiscZeroError;

#[no_mangle]
pub extern "C" fn submit_batch(){
    // This should be the risc0 Receipt for any Risc0 circuit used. 
    // The contract should compare the circuit ID against the expected one.
    let proof: Vec<u8> = runtime::get_named_arg(RUNTIME_ARG_PROOF);
    // deserialize and perform checks on journal
    let result: [u8;1] = risc0_verifier(proof);
    if result != [1u8]{
        runtime::revert(RiscZeroError::InvalidProof);
    }
}

#[no_mangle]
pub extern "C" fn call(){
    let entry_points = {
        let mut entry_points = EntryPoints::new();
        let submit_batch = EntryPoint::new(
            "submit_batch",
            vec![Parameter::new(RUNTIME_ARG_PROOF, CLType::Key)],
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