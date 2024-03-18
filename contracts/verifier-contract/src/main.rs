#![no_std]
#![no_main]
extern crate alloc;
use alloc::{string::ToString, vec, vec::Vec};
use casper_contract::{
    contract_api::{risc0::{self, risc0_verifier}, runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
mod constants;
use constants::{KAIROS_VERIFIER_CONTRACT, KAIROS_VERIFIER_CONTRACT_NAME, KAIROS_VERIFIER_CONTRACT_PACKAGE, RUNTIME_ARG_PROOF, DELTA_TREE_HEIGHT_COUNTER, DELTA_TREE_HISTORY_DICT};
use casper_types::{
    EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, contracts::NamedKeys,
    AccessRights, ApiError, CLType, CLValue, Key, Parameter, URef, U512, ContractHash, bytesrepr::ToBytes, bytesrepr::FromBytes, bytesrepr::Bytes
};
mod error;
use error::RiscZeroError;
use kairos_risc0_types::RiscZeroProof;
use bincode;
//use risc0_zkvm::Receipt;

#[no_mangle]
pub extern "C" fn submit_delta_tree_batch(){
    /* Delta Tree Initialization
        Delta tree updates / submissions occur from a genesis tree of form:

            let mut tree: KairosDeltaTree = KairosDeltaTree{
                zero_node: hash_bytes(vec![0;32]),
                zero_levels: Vec::new(),
                filled: vec![vec![], vec![], vec![], vec![], vec![]],
                root: None,
                index: 0,
                depth: 5
            };

        For the ith tree the verifying client will need to know the proof
        at i, the new leaf and the tree at i - 1. The leaf is the hash of the Batch inserted at i.
        For the insertion at index 0 in the contract dict, the default tree will be at
        i - 1, e.g. at -1.
    */


    // This should be the risc0 Receipt for any Risc0 circuit used. 
    // The contract should compare the circuit ID against the expected one.
    let proof: Vec<u8> = runtime::get_named_arg(RUNTIME_ARG_PROOF);
    
    // Todo: Perform checks on journal (match the guest program ID)
    let deserialized_proof: RiscZeroProof = bincode::deserialize(&proof).unwrap();
    let program_id: [u32;8] = deserialized_proof.program_id.try_into().unwrap();

    // deserialization is not recommended in wasm since kairos-risc0-types feature "kairos-delta-tree" is required
    // currently the kairos-delta-tree is not suitable for a no-std environment. Therefore the 
    // journal will be stored instead. Types from risc0-zkvm are also not importable in this environment,
    // due to dependency on floating points.
    // A host-function could be used to deserialize the struct, but storing the serialized proof is an alternative with 
    // both tradeoffs and benefits.

    // IGNORE!
    // let receipt: Receipt = bincode::deserialize(&deserialized_proof.receipt_serialized).unwrap();
    // let journal: CircuitJournal = receipt.journal.decode::<CircuitJournal>().unwrap();
    // let new_delta_tree_snapshot_serialized: Vec<u8> = bincode::serialize(&journal.output).unwrap();
    // let snapshot_journal_serialized: Vec<u8> = bincode::serialize(&receipt.journal).unwrap();
    // let snapshot_journal_serialized: Vec<u8> = vec![];
    
    ////////////////////////////////////////
    // insert match guest program ID here //
    ////////////////////////////////////////
    
    let result: [u8;1] = risc0_verifier(&proof);
    if result != [1u8]{
        runtime::revert(RiscZeroError::InvalidProof);
    };
    let delta_tree_height_counter_uref: URef = runtime::get_key(DELTA_TREE_HEIGHT_COUNTER)
        .unwrap()
        .into_uref()
        .unwrap();
    let mut delta_tree_height_counter_value: u64 = storage::read(delta_tree_height_counter_uref)
        .unwrap()
        .unwrap();
    // Insert the encoded CircuitJournal at delta_tree_height
    let delta_tree_history_dict_uref: URef = runtime::get_key(DELTA_TREE_HISTORY_DICT)
        .unwrap()
        .into_uref()
        .unwrap();
    storage::dictionary_put::<Vec<u8>>(
        delta_tree_history_dict_uref,
        &delta_tree_height_counter_value.to_string(),
        proof
    );
    // update the counter for the delta tree height
    delta_tree_height_counter_value += 1u64;
    storage::write(
        delta_tree_height_counter_uref,
        delta_tree_height_counter_value
    );
}

#[no_mangle]
pub extern "C" fn call(){
    let entry_points = {
        let mut entry_points = EntryPoints::new();
        let submit_batch = EntryPoint::new(
            "submit_delta_tree_batch",
            vec![Parameter::new(RUNTIME_ARG_PROOF, CLType::Key)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );
        entry_points.add_entry_point(submit_batch);
        entry_points
    };
    let mut named_keys = NamedKeys::new();
    // dictionary to store the root history of the delta tree
    let delta_tree_history_dict = storage::new_dictionary(DELTA_TREE_HISTORY_DICT).unwrap();
    named_keys.insert(DELTA_TREE_HISTORY_DICT.to_string(), delta_tree_history_dict.into());
    // counter to track the height of the delta tree
    let delta_tree_height_counter = storage::new_uref(u64::from(0u8));
    named_keys.insert(DELTA_TREE_HEIGHT_COUNTER.to_string(), delta_tree_height_counter.into());
    let (contract_hash, _) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(KAIROS_VERIFIER_CONTRACT.to_string()),
        Some(KAIROS_VERIFIER_CONTRACT_PACKAGE.to_string()),
    );
    let contract_hash_key = Key::from(contract_hash);
    runtime::put_key(KAIROS_VERIFIER_CONTRACT_NAME, contract_hash_key);
}