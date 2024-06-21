#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
#![no_std] // std support is experimental

use kairos_circuit_logic::ProofInputs;
use risc0_zkvm::{guest::env, serde::WordWrite};

risc0_zkvm::guest::entry!(main);

fn main() {
    let proof_inputs: ProofInputs = env::read();

    let output = proof_inputs
        .run_batch_proof_logic()
        .unwrap()
        .borsh_serialize()
        .unwrap();

    env::journal()
        .write_words(&[output.len().try_into().unwrap()])
        .unwrap();
    env::journal().write_padded_bytes(&output).unwrap();
}
