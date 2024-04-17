use std::sync::Arc;

use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::routing::TypedPath;
use tracing::*;

use kairos_tx::asn::{SigningPayload, TransactionBody};

use crate::{
    routes::PayloadBody,
    state::{
        transactions::{Signed, Transaction},
        BatchStateManager, TrieStateThreadMsg,
    },
    AppErr,
};

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/deposit")]
pub struct DepositPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn deposit_handler(
    _: DepositPath,
    state: State<Arc<BatchStateManager>>,
    Json(body): Json<PayloadBody>,
) -> Result<(), AppErr> {
    tracing::info!("parsing transaction data");
    let signing_payload: SigningPayload =
        body.payload.as_slice().try_into().context("payload err")?;
    let deposit = match signing_payload.body {
        TransactionBody::Deposit(deposit) => deposit.try_into().context("decoding deposit")?,
        _ => {
            return Err(AppErr::set_status(
                anyhow!("invalid transaction type"),
                StatusCode::BAD_REQUEST,
            ))
        }
    };

    let public_key = body.public_key;
    let epoch = signing_payload.epoch.try_into().context("decoding epoch")?;
    let nonce = signing_payload.nonce.try_into().context("decoding nonce")?;

    state
        .queued_transactions
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
