use std::sync::Arc;

use axum::{extract::State, Json};
use axum_extra::routing::TypedPath;
use tracing::*;

use kairos_circuit_logic::transactions::{KairosTransaction, L1Deposit};

use crate::{state::BatchStateManager, AppErr};

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/deposit")]
pub struct DepositPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn deposit_handler(
    _: DepositPath,
    state: State<Arc<BatchStateManager>>,
    Json(deposit): Json<L1Deposit>,
) -> Result<(), AppErr> {
    state
        .enqueue_transaction(KairosTransaction::Deposit(deposit))
        .await
}
