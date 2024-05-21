#![no_std]
extern crate alloc;

use alloc::{format, string::String};

use kairos_circuit_logic::ProofOutputs;

use methods::PROVE_BATCH_ID;
use risc0_zkvm::Receipt;

pub fn verify_execution(receipt: Receipt) -> Result<ProofOutputs, String> {
    receipt
        .verify(PROVE_BATCH_ID)
        .map_err(|e| format!("Error in risc0_zkvm verify: {e}"))?;

    let proof_outputs: ProofOutputs = receipt
        .journal
        .decode()
        .map_err(|e| format!("Error in risc0_zkvm decode: {e}"))?;

    Ok(proof_outputs)
}
