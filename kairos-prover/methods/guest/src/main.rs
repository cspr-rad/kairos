#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
#![no_std] // std support is experimental

use kairos_circuit_logic::ProofInputs;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
    let proof_inputs: ProofInputs = env::read();

    let output = proof_inputs.run_batch_proof_logic().unwrap();

    env::commit(&output);
}
