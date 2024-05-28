use axum::{http::StatusCode, Json};
use axum_extra::routing::{RouterExt, TypedPath};
use kairos_circuit_logic::{ProofInputs, ProofOutputs};

// These constants represent the RISC-V ELF and the image ID generated by risc0-build.
// The ELF is used for proving and the ID is used for verification.
use methods::{PROVE_BATCH_ELF, PROVE_BATCH_ID};
use risc0_zkvm::{ExecutorEnv, Receipt};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let socket_addr = std::env::var("KAIROS_PROVER_SERVER_SOCKET_ADDR")
        .expect("Failed to fetch environment variable KAIROS_PROVER_SERVER_SOCKET_ADDR");
    let socket_addr = socket_addr
        .parse::<std::net::SocketAddr>()
        .expect("Failed to parse KAIROS_SERVER_SOCKET_ADDR");

    let app = axum::Router::new()
        .typed_post(prove_batch_route)
        .with_state(());

    tracing::info!("starting http server on `{}`", socket_addr);
    let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();
    tracing::info!("listening on `{}`", socket_addr);
    axum::serve(listener, app).await.unwrap()
}

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/prove/batch")]
pub struct ProveBatch;

pub async fn prove_batch_route(
    _: ProveBatch,
    proof_inputs: Json<ProofInputs>,
) -> Result<Json<(ProofOutputs, Receipt)>, (StatusCode, String)> {
    let proof = tokio::task::spawn_blocking(move || prove_execution(proof_inputs.0))
        .await
        .map_err(|e| {
            let e = format!("Error while joining proving task: {e}");
            tracing::error!(e);
            (StatusCode::INTERNAL_SERVER_ERROR, e)
        })?;

    let proof = proof.map_err(|e| {
        let e = format!("Error while proving batch: {e}");
        tracing::error!(e);
        (StatusCode::INTERNAL_SERVER_ERROR, e)
    })?;

    Ok(Json(proof))
}

fn prove_execution(
    proof_inputs: kairos_circuit_logic::ProofInputs,
) -> Result<(ProofOutputs, Receipt), String> {
    let env = ExecutorEnv::builder()
        .write(&proof_inputs)
        .map_err(|e| format!("Error in ExecutorEnv builder write: {e}"))?
        .build()
        .map_err(|e| format!("Error in ExecutorEnv builder build: {e}"))?;

    let receipt = risc0_zkvm::default_prover()
        .prove(env, PROVE_BATCH_ELF)
        .map_err(|e| format!("Error in risc0_zkvm prove: {e}"))?;

    let proof_outputs = receipt
        .journal
        .decode()
        .map_err(|e| format!("Error in receipt journal decode: {e}"))?;

    receipt
        .verify(PROVE_BATCH_ID)
        .map_err(|e| format!("Error in risc0_zkvm verify: {e}"))?;

    Ok((proof_outputs, receipt))
}

pub fn cfg_disable_dev_mode_feature() {
    if cfg!(feature = "disable-dev-mode") {
        std::env::set_var("RISC0_DEV_MODE", "0");
    }
}

#[cfg(test)]
mod tests {
    use std::{rc::Rc, time::Instant};

    use kairos_trie::{stored::memory_db::MemoryDb, TrieRoot};
    use proptest::prelude::*;

    use kairos_circuit_logic::{
        account_trie::{test_logic::test_prove_batch, Account},
        transactions::{
            arbitrary::RandomBatches, KairosTransaction, L1Deposit, Signed, Transfer, Withdraw,
        },
    };

    use crate::cfg_disable_dev_mode_feature;

    #[test]
    fn test_prove_simple_batches() {
        cfg_disable_dev_mode_feature();

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

        test_prove_batch(
            TrieRoot::Empty,
            Rc::new(MemoryDb::<Account>::empty()),
            batches,
            |proof_inputs| {
                Ok(crate::prove_execution(proof_inputs)
                    .map(|(proof_outputs, _)| proof_outputs)
                    .expect("Failed to prove execution"))
            },
        );
    }

    #[test_strategy::proptest(ProptestConfig::default(), cases = 1)]
    fn proptest_prove_batches(
        #[any(max_batch_size = 5, max_batch_count = 3, max_initial_l2_accounts = 30)]
        args: RandomBatches,
    ) {
        cfg_disable_dev_mode_feature();
        let batches = args.filter_success();

        proptest::prop_assume!(!batches.is_empty());

        test_prove_batch(args.initial_trie, args.trie_db, batches, |proof_inputs| {
            eprintln!(
                "Proving batch of size: {}, over trie of {} accounts.",
                proof_inputs.transactions.len(),
                args.initial_state.l2.len()
            );

            let timestamp = Instant::now();
            let (proof_outputs, _) =
                crate::prove_execution(proof_inputs).expect("Failed to prove execution");

            let elapsed = timestamp.elapsed().as_secs_f64();
            eprintln!("Proved batch: {elapsed}s");

            Ok(proof_outputs)
        })
    }
}
