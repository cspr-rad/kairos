use std::time::Instant;

use axum::{http::StatusCode, Json};
use axum_extra::routing::{RouterExt, TypedPath};
use kairos_circuit_logic::{ProofInputs, ProofOutputs};

// These constants represent the RISC-V ELF and the image ID generated by risc0-build.
// The ELF is used for proving and the ID is used for verification.
use methods::{PROVE_BATCH_ELF, PROVE_BATCH_ID};
use risc0_zkvm::{ExecutorEnv, Prover, Receipt};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

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
    let timestamp = Instant::now();

    let env = ExecutorEnv::builder()
        .write(&proof_inputs)
        .map_err(|e| format!("Error in ExecutorEnv builder write: {e}"))?
        .build()
        .map_err(|e| format!("Error in ExecutorEnv builder build: {e}"))?;

    let receipt = match (cfg!(feature = "cuda"), cfg!(feature = "metal")) {
        (true, true) => panic!("Cannot enable both CUDA and Metal features"),
        #[cfg(feature = "cuda")]
        (true, false) => risc0_zkvm::LocalProver::new("local metal").prove(env, PROVE_BATCH_ELF),
        #[cfg(feature = "metal")]
        (false, true) => risc0_zkvm::LocalProver::new("local cuda").prove(env, PROVE_BATCH_ELF),
        _ => risc0_zkvm::ExternalProver::new("ipc r0vm", env!("RISC0_R0VM_PATH"))
            .prove(env, PROVE_BATCH_ELF),
    }
    .map_err(|e| format!("Error in risc0_zkvm prove: {e}"))?;

    tracing::info!("Proved batch: {}s", timestamp.elapsed().as_secs_f64());

    let timestamp = Instant::now();

    let proof_outputs = kairos_verifier_risc0_lib::verify_execution(&receipt, PROVE_BATCH_ID)?;

    tracing::info!("Verified batch: {}s", timestamp.elapsed().as_secs_f64());

    Ok((proof_outputs, receipt))
}

pub fn test_setup() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .try_init();

    if cfg!(feature = "disable-dev-mode") {
        std::env::set_var("RISC0_DEV_MODE", "0");
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use kairos_trie::{stored::memory_db::MemoryDb, TrieRoot};
    use proptest::prelude::*;

    use kairos_circuit_logic::{
        account_trie::{test_logic::test_prove_batch, Account},
        transactions::{
            arbitrary::RandomBatches, KairosTransaction, L1Deposit, Signed, Transfer, Withdraw,
        },
    };

    use crate::test_setup;

    #[test]
    fn test_prove_simple_batches() {
        test_setup();

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

    #[test_strategy::proptest(ProptestConfig::default(), cases = if cfg!(feature = "disable-dev-mode") { 2 } else { 40 })]
    fn proptest_prove_batches(
        #[any(batch_size = 1..=4, batch_count = 2..=4, initial_l2_accounts = 10_000..=100_000)]
        args: RandomBatches,
    ) {
        test_setup();
        let batches = args.filter_success();

        proptest::prop_assume!(batches.len() >= 2);

        test_prove_batch(args.initial_trie, args.trie_db, batches, |proof_inputs| {
            tracing::info!(
                "Proving batch of size: {}, over trie of {} accounts.",
                proof_inputs.transactions.len(),
                args.initial_state.l2.len()
            );

            let (proof_outputs, _) =
                crate::prove_execution(proof_inputs).expect("Failed to prove execution");

            Ok(proof_outputs)
        })
    }
}
