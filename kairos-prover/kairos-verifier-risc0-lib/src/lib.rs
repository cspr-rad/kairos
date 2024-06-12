#![no_std]

/// The risc0 image_id of the batch update circuit program.
pub const BATCH_CIRCUIT_PROGRAM_HASH: [u32; 8] = [
    1854829392, 1056154338, 1061691258, 418817515, 2587812152, 317072623, 3636186866, 3117322684,
];

#[cfg(feature = "verifier")]
pub mod verifier {
    extern crate alloc;

    use alloc::{format, string::String};

    use kairos_circuit_logic::ProofOutputs;

    use risc0_zkvm::Receipt;

    pub fn verify_execution(receipt: &Receipt) -> Result<ProofOutputs, String> {
        receipt
            .verify(crate::BATCH_CIRCUIT_PROGRAM_HASH)
            .map_err(|e| format!("Error in risc0_zkvm verify: {e}"))?;

        let proof_outputs = ProofOutputs::rkyv_deserialize(&receipt.journal.bytes)?;

        Ok(proof_outputs)
    }
}
