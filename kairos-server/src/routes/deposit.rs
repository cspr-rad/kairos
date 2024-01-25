use std::ops::Deref;

use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::routing::TypedPath;
use serde::{Deserialize, Serialize};

use crate::{state::LockedBatchState, AppErr, PublicKey};

#[derive(TypedPath)]
#[typed_path("/deposit")]
pub struct DepositPath;

#[derive(Serialize, Deserialize)]
pub struct Deposit {
    pub public_key: PublicKey,
    pub amount: u64,
}

pub async fn handler(
    _: DepositPath,
    state: State<LockedBatchState>,
    Json(Deposit { public_key, amount }): Json<Deposit>,
) -> Result<(), AppErr> {
    tracing::info!("TODO: verifying deposit");

    tracing::info!("TODO: adding deposit to batch");

    let mut state = state.deref().write().await;
    let account = state.balances.entry(public_key.clone());

    let prior_balance = account.or_insert(0);
    let updated_balance = prior_balance.checked_add(amount).ok_or_else(|| {
        AppErr::set_status(
            anyhow!("deposit would overflow account"),
            StatusCode::CONFLICT,
        )
    })?;

    tracing::info!(
        "Updated account public_key={} balance={}",
        public_key,
        updated_balance
    );

    Ok(())
}
