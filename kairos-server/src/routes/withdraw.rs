use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::routing::TypedPath;
use tracing::*;

use crate::{state::AppState, state::LockedBatchState, AppErr, PublicKey};
use kairos_tx::asn::{SigningPayload, TransactionBody};

use crate::routes::PayloadBody;

#[derive(Debug, TypedPath)]
#[typed_path("/api/v1/withdraw")]
pub struct WithdrawPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn withdraw_handler(
    _: WithdrawPath,
    State(state): State<AppState>,
    Json(body): Json<PayloadBody>,
) -> Result<(), AppErr> {
    tracing::info!("parsing transaction data");
    let signing_payload: SigningPayload =
        body.payload.as_slice().try_into().context("payload err")?;
    let withdrawal = match signing_payload.body {
        TransactionBody::Withdrawal(withdrawal) => withdrawal,
        _ => {
            return Err(AppErr::set_status(
                anyhow!("invalid transaction type"),
                StatusCode::BAD_REQUEST,
            ))
        }
    };
    let amount = u64::try_from(withdrawal.amount).context("invalid amount")?;
    let public_key = body.public_key;

    tracing::info!("TODO: verifying withdrawal signature");

    tracing::info!("verifying withdrawal sender has sufficient funds");
    check_sender_funds(&state.batch_state, &public_key, amount).await?;

    tracing::info!("TODO: adding withdrawal to batch");

    let mut batch_state = state.batch_state.write().await;
    let from_balance = batch_state.balances.get_mut(&public_key).ok_or_else(|| {
        AppErr::set_status(
            anyhow!(
                "Sender no longer has an account.
                The sender just removed all their funds."
            ),
            StatusCode::CONFLICT,
        )
    })?;

    let updated_balance = from_balance.checked_sub(amount).ok_or_else(|| {
        AppErr::set_status(
            anyhow!(
                "Sender no longer has sufficient funds, balance={}, withdrawal_amount={}.
                The sender just moved their funds in a concurrent request",
                from_balance,
                amount
            ),
            StatusCode::CONFLICT,
        )
    })?;

    *from_balance = updated_balance;

    if updated_balance == 0 {
        batch_state.balances.remove(&public_key);
    }

    tracing::info!(
        "Updated account public_key={:?} balance={}",
        public_key,
        updated_balance
    );

    Ok(())
}

async fn check_sender_funds(
    state: &LockedBatchState,
    public_key: &PublicKey,
    amount: u64,
) -> Result<(), AppErr> {
    let state = state.read().await;
    let from_balance = state.balances.get(public_key).ok_or_else(|| {
        AppErr::set_status(anyhow!("Withdrawer has no account."), StatusCode::CONFLICT)
    })?;

    if *from_balance < amount {
        return Err(AppErr::set_status(
            anyhow!(
                "Withdrawer has insufficient funds, balance={}, withdrawal_amount={}.",
                from_balance,
                amount
            ),
            StatusCode::FORBIDDEN,
        ));
    }

    Ok(())
}
