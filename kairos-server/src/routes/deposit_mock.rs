use axum::{extract::State, Json};
use axum_extra::routing::TypedPath;
use tracing::*;

use kairos_circuit_logic::transactions::{KairosTransaction, L1Deposit};

#[cfg(feature="database")]
use kairos_data::transaction as db;

use crate::{state::ServerState, AppErr};

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/deposit-mock")]
pub struct MockDepositPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn deposit_mock_handler(
    _: MockDepositPath,
    state: State<ServerState>,
    Json(deposit): Json<L1Deposit>,
) -> Result<(), AppErr> {
    tracing::info!("parsing transaction data");

    let deposit = KairosTransaction::Deposit(deposit);
    #[cfg(feature="database")]
    db::insert(state.pool.clone(), deposit.clone()).await?;
    state
        .batch_state_manager
        .enqueue_transaction(deposit)
        .await
}
