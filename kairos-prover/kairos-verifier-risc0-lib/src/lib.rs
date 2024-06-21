#![no_std]

/// The risc0 image_id of the batch update circuit program.
/// To update this you must rebuild the circuit with `cargo risczero build`, which will use docker
/// to reproducibly build the circuit.
/// Then you must copy the output ELF to `kairos-prover/methods/prove_batch_bin`
/// and run `cargo build` which will output the new hash.
pub const BATCH_CIRCUIT_PROGRAM_HASH: [u32; 8] = [
    1933896867, 724949015, 2801151840, 3598218816, 678156014, 3617865613, 2087653082, 1179432235,
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

    pub fn verify_execution_of_any_program_with_error_hooks<E>(
        receipt: &Receipt,
        image_id: [u32; 8],
        verify_err: impl FnOnce(String) -> E,
        deserialize_err: impl FnOnce(String) -> E,
    ) -> Result<ProofOutputs, E> {
        receipt
            .verify(image_id)
            .map_err(|e| verify_err(format!("Error in risc0_zkvm verify: {e}")))?;

        let proof_outputs =
            ProofOutputs::rkyv_deserialize(&receipt.journal.bytes).map_err(|e| {
                deserialize_err(format!("Error in rkyv deserialize of proof outputs: {e}"))
            })?;

        Ok(proof_outputs)
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
