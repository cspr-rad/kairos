#![cfg_attr(target_os = "zkvm", no_main)]

#[cfg(target_os = "zkvm")]
sp1_zkvm::entrypoint!(main);

use kairos_circuit_logic::ProofInputs;

pub fn main() {
    let proof_inputs: ProofInputs = sp1_zkvm::io::read();

    let output = proof_inputs
        .run_batch_proof_logic()
        .unwrap()
        .borsh_serialize()
        .unwrap();

    sp1_zkvm::io::commit_slice(&output);
}
