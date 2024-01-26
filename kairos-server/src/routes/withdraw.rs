use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::routing::TypedPath;
use serde::{Deserialize, Serialize};
use tracing::*;

use crate::{state::LockedBatchState, AppErr, PublicKey};

#[derive(Debug, TypedPath)]
#[typed_path("/api/v1/withdraw")]
pub struct WithdrawPath;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Withdrawal {
    pub public_key: PublicKey,
    pub signature: String,
    pub amount: u64,
}

#[instrument(level = "trace", skip(state), ret)]
pub async fn withdraw_handler(
    _: WithdrawPath,
    State(state): State<LockedBatchState>,
    Json(withdrawal): Json<Withdrawal>,
) -> Result<(), AppErr> {
    tracing::info!("TODO: verifying withdrawal signature");

    tracing::info!("verifying withdrawal sender has sufficient funds");
    check_sender_funds(&state, &withdrawal).await?;

    tracing::info!("TODO: adding withdrawal to batch");

    let mut state = state.write().await;
    let from_balance = state
        .balances
        .get_mut(&withdrawal.public_key)
        .ok_or_else(|| {
            AppErr::set_status(
                anyhow!(
                    "Sender no longer has an account.
                The sender just removed all their funds."
                ),
                StatusCode::CONFLICT,
            )
        })?;

    let updated_balance = from_balance.checked_sub(withdrawal.amount).ok_or_else(|| {
        AppErr::set_status(
            anyhow!(
                "Sender no longer has sufficient funds, balance={}, withdrawal_amount={}.
                The sender just moved their funds in a concurrent request",
                from_balance,
                withdrawal.amount
            ),
            StatusCode::CONFLICT,
        )
    })?;

    *from_balance = updated_balance;

    if updated_balance == 0 {
        state.balances.remove(&withdrawal.public_key);
    }

    tracing::info!(
        "Updated account public_key={} balance={}",
        withdrawal.public_key,
        updated_balance
    );

    Ok(())
}

async fn check_sender_funds(
    state: &LockedBatchState,
    withdrawal: &Withdrawal,
) -> Result<(), AppErr> {
    let state = state.read().await;
    let from_balance = state.balances.get(&withdrawal.public_key).ok_or_else(|| {
        AppErr::set_status(anyhow!("Withdrawer has no account."), StatusCode::CONFLICT)
    })?;

    if *from_balance < withdrawal.amount {
        return Err(AppErr::set_status(
            anyhow!(
                "Withdrawer has insufficient funds, balance={}, withdrawal_amount={}.",
                from_balance,
                withdrawal.amount
            ),
            StatusCode::FORBIDDEN,
        ));
    }

    Ok(())
}
