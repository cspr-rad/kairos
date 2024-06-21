#![no_std]

/// The risc0 image_id of the batch update circuit program.
/// To update this you must rebuild the circuit with `cargo risczero build`, which will use docker
/// to reproducibly build the circuit.
/// Then you must copy the output ELF to `kairos-prover/methods/prove_batch_bin`
/// and run `cargo build` which will output the new hash.
pub const BATCH_CIRCUIT_PROGRAM_HASH: [u32; 8] = [
    2249819926, 1807275128, 879420467, 753150136, 3885109892, 1252737579, 1362575552, 43533945,
];

#[cfg(feature = "verifier")]
pub mod verifier {
    extern crate alloc;

    use alloc::{borrow::ToOwned, format, string::String};

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

        let (len, borsh_bytes) = match receipt.journal.bytes.as_slice() {
            [a, b, c, d, rest @ ..] => (u32::from_le_bytes([*a, *b, *c, *d]), rest),
            _ => return Err("No bytes in journal".to_owned()),
        };

        let proof_outputs = ProofOutputs::borsh_deserialize(&borsh_bytes[..len as usize])
            .map_err(|e| format!("Error in borsh deserialize: {e}"))?;

        Ok(proof_outputs)
    }
}
