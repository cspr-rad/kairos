use std::ops::Deref;

use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::routing::TypedPath;
use tracing::*;

use crate::{routes::PayloadBody, state::AppState, AppErr};
use kairos_tx::asn::{SigningPayload, TransactionBody};

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/deposit")]
pub struct DepositPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn deposit_handler(
    _: DepositPath,
    state: State<AppState>,
    Json(body): Json<PayloadBody>,
) -> Result<(), AppErr> {
    tracing::info!("parsing transaction data");
    let signing_payload: SigningPayload =
        body.payload.as_slice().try_into().context("payload err")?;
    let deposit = match signing_payload.body {
        TransactionBody::Deposit(deposit) => deposit,
        _ => {
            return Err(AppErr::set_status(
                anyhow!("invalid transaction type"),
                StatusCode::BAD_REQUEST,
            ))
        }
    };
    let amount = u64::try_from(deposit.amount).context("invalid amount")?;
    let public_key = body.public_key;

    tracing::info!("TODO: verifying deposit");

    tracing::info!("TODO: adding deposit to batch");

    let mut batch_state = state.batch_state.deref().write().await;
    let account = batch_state.balances.entry(public_key.clone());

    let balance = account.or_insert(0);
    let updated_balance = balance.checked_add(amount).ok_or_else(|| {
        AppErr::set_status(
            anyhow!("deposit would overflow account"),
            StatusCode::CONFLICT,
        )
    })?;

    *balance = updated_balance;

    tracing::info!(
        "Updated account public_key={:?} balance={}",
        public_key,
        updated_balance
    );

    Ok(())
}
