use std::fmt::Display;

use axum::{http::StatusCode, Json};
use axum_extra::routing::{RouterExt, TypedPath};
use dotenvy::dotenv;
use kairos_circuit_logic::{ProofInputs, ProofOutputs};

pub trait Prover: 'static {
    type Error: 'static + Send + Display;
    type ProofSystemData: 'static + Send + serde::Serialize + for<'de> serde::Deserialize<'de>;
    fn prove_execution(
        proof_inputs: ProofInputs,
    ) -> Result<Proof<Self::ProofSystemData>, Self::Error>;
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Proof<T> {
    pub logical_outputs: ProofOutputs,
    pub proof_system_data: T,
}

/// A prover that executes the proof logic natively.
/// Semantically a successful execution of a batch is proof for anyone who trusts server.
pub struct NativeTrustedProver;

impl Prover for NativeTrustedProver {
    type Error = String;
    type ProofSystemData = ();

    fn prove_execution(proof_inputs: ProofInputs) -> Result<Proof<()>, Self::Error> {
        let logical_outputs = proof_inputs
            .run_batch_proof_logic()
            .map_err(|e| format!("Failed to run batch proof logic: {e}"))?;
        Ok(Proof {
            logical_outputs,
            proof_system_data: (),
        })
    }
}

pub async fn run_server<P: Prover>() {
    let _ = dotenv();

    let socket_addr = std::env::var("KAIROS_PROVER_SERVER_SOCKET_ADDR")
        .expect("Failed to fetch environment variable KAIROS_PROVER_SERVER_SOCKET_ADDR");
    let socket_addr = socket_addr
        .parse::<std::net::SocketAddr>()
        .expect("Failed to parse KAIROS_SERVER_SOCKET_ADDR");

    let app = router::<P>().with_state(());

    tracing::info!("starting http server on `{}`", socket_addr);
    let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();
    tracing::info!("listening on `{}`", socket_addr);
    axum::serve(listener, app).await.unwrap();
}

pub fn router<P: Prover>() -> axum::Router {
    axum::Router::new().typed_post(prove_batch::<P>)
}

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/prove/batch")]
pub struct ProveBatch;

pub async fn prove_batch<P: Prover>(
    _: ProveBatch,
    proof_inputs: Json<ProofInputs>,
) -> Result<Json<Proof<P::ProofSystemData>>, (StatusCode, String)> {
    let proof = tokio::task::spawn_blocking(move || P::prove_execution(proof_inputs.0))
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

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use kairos_circuit_logic::{
        account_trie::{Account, AccountTrie},
        transactions::{KairosTransaction, L1Deposit, Signed, Transfer, Withdraw},
    };
    use kairos_trie::{stored::memory_db::MemoryDb, DigestHasher, TrieRoot};

    use crate::{NativeTrustedProver, Prover};

    #[test]
    fn test_prove_batch() {
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

        let db = Rc::new(MemoryDb::<Account>::empty());
        let mut prior_root_hash = TrieRoot::default();

        for batch in batches.into_iter() {
            let mut account_trie = AccountTrie::new_try_from_db(db.clone(), prior_root_hash)
                .expect("Failed to create account trie");
            account_trie
                .apply_batch(batch.iter().cloned())
                .expect("Failed to apply batch");

            let new_root_hash = account_trie
                .txn
                .commit(&mut DigestHasher::<sha2::Sha256>::default())
                .expect("Failed to commit transaction");

            let trie_snapshot = account_trie.txn.build_initial_snapshot();

            let proof_inputs = kairos_circuit_logic::ProofInputs {
                transactions: batch.into_boxed_slice(),
                trie_snapshot,
            };

            let _proof = NativeTrustedProver::prove_execution(proof_inputs)
                .expect("Failed to prove execution");

            prior_root_hash = new_root_hash;
        }
    }
}
