use std::ops::Deref;

use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::routing::TypedPath;
use tracing::*;

use kairos_tx::asn::{SigningPayload, TransactionBody};

use crate::routes::PayloadBody;
use crate::state::transactions::{Deposit, Signed, Transaction};
use crate::state::TrieStateThreadMsg;
use crate::{state::LockedBatchState, AppErr};

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/deposit")]
pub struct DepositPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn deposit_handler(
    _: DepositPath,
    state: State<LockedBatchState>,
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
    let signed = Signed {
        public_key: body.public_key,
        epoch: signing_payload.epoch.try_into().context("decoding epoch")?,
        nonce: signing_payload.nonce.try_into().context("decoding nonce")?,
        transaction: Deposit::try_from(deposit).context("decoding deposit")?,
    };
    let amount = signed.transaction.amount;
    let public_key = &signed.public_key;

    if amount == 0 {
        return Err(AppErr::set_status(
            anyhow!("deposit amount must be greater than 0"),
            StatusCode::BAD_REQUEST,
        ));
    }

    // TODO are deposits connected to a specific epoch?

    tracing::info!("TODO: verifying deposit");

    tracing::info!("TODO: adding deposit to batch");

    let mut state = state.deref().write().await;
    let account = state.balances.entry(public_key.clone());

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
    tracing::info!("queuing deposit transaction");

    let queued_txn = state.queued_transactions.clone();
    // Relase the write lock before queuing the transaction
    drop(state);

    let Signed {
        public_key,
        epoch,
        nonce,
        transaction: deposit,
    } = signed;
    queued_txn
        .send(TrieStateThreadMsg::Transaction(Signed {
            public_key,
            epoch,
            nonce,
            transaction: Transaction::Deposit(deposit),
        }))
        .await
        .context("sending transaction to trie thread")?;

    Ok(())
}
