#![no_std]

/// The risc0 image_id of the batch update circuit program.
pub const BATCH_CIRCUIT_PROGRAM_HASH: [u32; 8] = [
    126000422, 3712867871, 1167515693, 1753452751, 2840572210, 3994253283, 1614161387, 3743629498,
];

#[cfg(feature = "verifier")]
pub mod verifier {
    extern crate alloc;

    use alloc::{format, string::String};

    pub use kairos_circuit_logic::ProofOutputs;

    pub use risc0_zkvm::Receipt;

    pub fn verify_execution(receipt: &Receipt) -> Result<ProofOutputs, String> {
        verify_execution_of_any_program(receipt, crate::BATCH_CIRCUIT_PROGRAM_HASH)
    }

    pub fn verify_execution_of_any_program(
        receipt: &Receipt,
        image_id: [u32; 8],
    ) -> Result<ProofOutputs, String> {
        receipt
            .verify(image_id)
            .map_err(|e| format!("Error in risc0_zkvm verify: {e}"))?;

        let proof_outputs = ProofOutputs::rkyv_deserialize(&receipt.journal.bytes)?;

        Ok(proof_outputs)
    }
}
