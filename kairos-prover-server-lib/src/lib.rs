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
            .expect("Failed to run batch proof logic");
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
