use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::routing::TypedPath;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{state::LockedBatchState, AppErr, PublicKey, Signature};

#[derive(TypedPath)]
#[typed_path("/api/v1/transfer")]
pub struct TransferPath;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Transfer {
    pub from: PublicKey,
    pub signature: Signature,
    pub to: PublicKey,
    pub amount: u64,
}

#[instrument(level = "trace", skip(state), ret)]
pub async fn transfer_handler(
    _: TransferPath,
    State(state): State<LockedBatchState>,
    Json(Transfer {
        from,
        signature,
        to,
        amount,
    }): Json<Transfer>,
) -> Result<(), AppErr> {
    if amount == 0 {
        return Err(AppErr::set_status(
            anyhow!("transfer amount must be greater than 0"),
            StatusCode::BAD_REQUEST,
        ));
    }

    tracing::info!("TODO: verifying transfer signature");

    // We pre-check this read-only to error early without acquiring the write lock.
    // This prevents a DoS attack exploiting the write lock.
    tracing::info!("verifying transfer sender has sufficient funds");
    check_sender_funds(&state, &from, amount, &to).await?;

    let mut state = state.write().await;
    let from_balance = state.balances.get_mut(&from).ok_or_else(|| {
        AppErr::set_status(
            anyhow!(
                "Sender no longer has an account.
                The sender just removed all their funds."
            ),
            StatusCode::CONFLICT,
        )
    })?;

    *from_balance = from_balance.checked_sub(amount).ok_or_else(|| {
        AppErr::set_status(
            anyhow!(
                "Sender no longer has sufficient funds, balance={}, transfer_amount={}.
                The sender just moved their funds in a concurrent request",
                from_balance,
                amount
            ),
            StatusCode::CONFLICT,
        )
    })?;

    let to_balance = state.balances.entry(to.clone()).or_insert_with(|| {
        tracing::info!("creating new account for receiver");
        0
    });

    *to_balance = to_balance.checked_add(amount).ok_or_else(|| {
        AppErr::set_status(anyhow!("Receiver balance overflow"), StatusCode::CONFLICT)
    })?;

    Ok(())
}

async fn check_sender_funds(
    state: &LockedBatchState,
    from: &PublicKey,
    amount: u64,
    to: &PublicKey,
) -> Result<(), AppErr> {
    let state = state.read().await;
    let from_balance = state.balances.get(from).ok_or_else(|| {
        AppErr::set_status(
            anyhow!("Sender does not have an account"),
            StatusCode::BAD_REQUEST,
        )
    })?;

    from_balance.checked_sub(amount).ok_or_else(|| {
        AppErr::set_status(
            anyhow!(
                "Sender does not have sufficient funds, balance={}, transfer_amount={}",
                from_balance,
                amount
            ),
            StatusCode::FORBIDDEN,
        )
    })?;

    let to_balance = state.balances.get(to).unwrap_or(&0);
    if to_balance.checked_add(amount).is_none() {
        return Err(AppErr::set_status(
            anyhow!("Receiver balance overflow"),
            StatusCode::CONFLICT,
        ));
    }

    Ok(())
}
