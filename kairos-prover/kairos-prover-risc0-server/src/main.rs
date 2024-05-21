use kairos_prover_server_lib::{run_server, Proof, Prover};
// These constants represent the RISC-V ELF and the image ID generated by risc0-build.
// The ELF is used for proving and the ID is used for verification.
use methods::{PROVE_BATCH_ELF, PROVE_BATCH_ID};
use risc0_zkvm::{ExecutorEnv, Receipt};

#[tokio::main]
async fn main() {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    // Start the server with the Risc0Prover
    run_server::<Risc0Prover>().await;
}

pub struct Risc0Prover;

impl Prover for Risc0Prover {
    type Error = String;
    type ProofSystemData = Receipt;

    fn prove_execution(
        proof_inputs: kairos_circuit_logic::ProofInputs,
    ) -> Result<Proof<Receipt>, String> {
        let env = ExecutorEnv::builder()
            .write(&proof_inputs)
            .map_err(|e| format!("Error in ExecutorEnv builder write: {e}"))?
            .build()
            .map_err(|e| format!("Error in ExecutorEnv builder build: {e}"))?;

        let receipt = risc0_zkvm::default_prover()
            .prove(env, PROVE_BATCH_ELF)
            .map_err(|e| format!("Error in risc0_zkvm prove: {e}"))?;

        let logical_outputs = receipt
            .journal
            .decode()
            .map_err(|e| format!("Error in receipt journal decode: {e}"))?;

        receipt
            .verify(PROVE_BATCH_ID)
            .map_err(|e| format!("Error in risc0_zkvm verify: {e}"))?;

        Ok(Proof {
            logical_outputs,
            proof_system_data: receipt,
        })
    }
}

#[cfg(test)]
mod tests {
    use kairos_circuit_logic::{
        account_trie::test_logic::test_prove_batch,
        transactions::{KairosTransaction, L1Deposit, Signed, Transfer, Withdraw},
    };
    use kairos_prover_server_lib::Prover;

    use crate::Risc0Prover;

    #[test]
    fn test_prove_simple_batches() {
        let alice_public_key = "alice_public_key".as_bytes().to_vec();
        let bob_public_key = "bob_public_key".as_bytes().to_vec();

        let batches = vec![
            vec![
                KairosTransaction::Deposit(L1Deposit {
                    recipient: alice_public_key.clone(),
                    amount: 10,
                }),
                KairosTransaction::Transfer(Signed {
                    public_key: alice_public_key.clone(),
                    transaction: Transfer {
                        recipient: bob_public_key.clone(),
                        amount: 5,
                    },
                    nonce: 0,
                }),
                KairosTransaction::Withdraw(Signed {
                    public_key: alice_public_key.clone(),
                    transaction: Withdraw { amount: 5 },
                    nonce: 1,
                }),
            ],
            vec![
                KairosTransaction::Transfer(Signed {
                    public_key: bob_public_key.clone(),
                    transaction: Transfer {
                        recipient: alice_public_key.clone(),
                        amount: 2,
                    },
                    nonce: 0,
                }),
                KairosTransaction::Withdraw(Signed {
                    public_key: bob_public_key.clone(),
                    transaction: Withdraw { amount: 3 },
                    nonce: 1,
                }),
                KairosTransaction::Withdraw(Signed {
                    public_key: alice_public_key.clone(),
                    transaction: Withdraw { amount: 2 },
                    nonce: 2,
                }),
            ],
        ];

        test_prove_batch(batches, |proof_inputs| {
            Ok(Risc0Prover::prove_execution(proof_inputs)
                .map(|o| o.logical_outputs)
                .expect("Failed to prove execution"))
        });
    }
}
