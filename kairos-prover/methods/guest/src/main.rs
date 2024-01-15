#![no_main]
#![no_std]
use risc0_zkvm::guest::env;
use serde_json;
risc0_zkvm::guest::entry!(main);
use kairos_types::transfer::{MerkleRoot, MockLayerOneState};
extern crate alloc;
use alloc::vec::Vec;

pub fn main() {
    let x: u32 = env::read();
    let y: u32 = env::read();
    if x != y {
        panic!("X != Y");
    }
    let new_root = MockLayerOneState {
        root: MerkleRoot { hash: Vec::new() },
    };
    let output = &serde_json::to_vec(&new_root).unwrap();
    env::commit(output);
}
