#![no_std]
extern crate alloc;

use alloc::{format, string::String};

use kairos_circuit_logic::ProofOutputs;

use risc0_zkvm::Receipt;

pub fn verify_execution(receipt: &Receipt, program_id: [u32; 8]) -> Result<ProofOutputs, String> {
    receipt
        .verify(program_id)
        .map_err(|e| format!("Error in risc0_zkvm verify: {e}"))?;

    let proof_outputs = ProofOutputs::rkyv_deserialize(&receipt.journal.bytes)?;

    Ok(proof_outputs)
}
