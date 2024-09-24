#![no_std]

#[allow(clippy::too_long_first_doc_paragraph)]
/// The risc0 image_id of the batch update circuit program.
/// To update this you must rebuild the circuit with `cargo risczero build`, which will use docker
/// to reproducibly build the circuit.
/// Then you must copy the output ELF to `kairos-prover/methods/prove_batch_bin`
/// and run `cargo build` which will output the new hash.
pub const BATCH_CIRCUIT_PROGRAM_HASH: [u32; 8] = [
    162636669, 3420136214, 482235019, 1624934074, 1024623463, 972941903, 3671681564, 1824242109,
];

#[cfg(feature = "verifier")]
pub mod verifier {
    extern crate alloc;

    use core::fmt::Display;

    use alloc::{
        format,
        string::{String, ToString},
    };

    pub use kairos_circuit_logic::ProofOutputs;

    pub use risc0_zkvm::Receipt;

    #[derive(Debug, Clone)]
    pub enum VerifyError {
        Ris0ZkvmVerifcationError(String),
        TooFewBytesInJournal {
            length: usize,
        },
        InvalidLengthInJournal {
            length_header: usize,
            real_length: usize,
        },
        BorshDeserializationError(String),
    }

    impl Display for VerifyError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                VerifyError::Ris0ZkvmVerifcationError(e) => write!(f, "Ris0ZkvmVerifcationError: {e}"),
                VerifyError::TooFewBytesInJournal { length } => {
                    write!(f, "TooFewBytesInJournal: {length}")
                }
                VerifyError::InvalidLengthInJournal {
                    length_header,
                    real_length,
                } => write!(
                    f,
                    "InvalidLengthInJournal: length_header: {length_header}, real_length: {real_length}"
                ),
                VerifyError::BorshDeserializationError(e) => {
                    write!(f, "BorshDeserializationError: {e}")
                }
            }
        }
    }

    impl From<VerifyError> for String {
        fn from(e: VerifyError) -> Self {
            e.to_string()
        }
    }

    pub fn verify_execution(receipt: &Receipt) -> Result<ProofOutputs, VerifyError> {
        verify_execution_of_any_program(receipt, crate::BATCH_CIRCUIT_PROGRAM_HASH)
    }

    pub fn verify_execution_of_any_program(
        receipt: &Receipt,
        image_id: [u32; 8],
    ) -> Result<ProofOutputs, VerifyError> {
        receipt
            .verify(image_id)
            .map_err(|e| VerifyError::Ris0ZkvmVerifcationError(format!("{e}")))?;

        let (len, borsh_bytes) = match receipt.journal.bytes.as_slice() {
            [a, b, c, d, rest @ ..] => (u32::from_le_bytes([*a, *b, *c, *d]), rest),
            _ => {
                return Err(VerifyError::TooFewBytesInJournal {
                    length: receipt.journal.bytes.len(),
                })
            }
        };

        let proof_outputs =
            ProofOutputs::borsh_deserialize(borsh_bytes.as_ref().get(..len as usize).ok_or(
                VerifyError::InvalidLengthInJournal {
                    length_header: len as usize,
                    real_length: borsh_bytes.len(),
                },
            )?)
            .map_err(|e| VerifyError::BorshDeserializationError(e.to_string()))?;

        Ok(proof_outputs)
    }
}
