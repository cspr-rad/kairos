use std::sync::Arc;

use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::routing::TypedPath;
use tracing::*;

use kairos_tx::asn::{SigningPayload, TransactionBody};

use crate::routes::PayloadBody;
use crate::state::transactions::{Signed, Transaction, Withdraw};
use crate::state::{BatchStateManager, TrieStateThreadMsg};
use crate::AppErr;

#[derive(Debug, TypedPath)]
#[typed_path("/api/v1/withdraw")]
pub struct WithdrawPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn withdraw_handler(
    _: WithdrawPath,
    State(state): State<Arc<BatchStateManager>>,
    Json(body): Json<PayloadBody>,
) -> Result<(), AppErr> {
    tracing::info!("parsing transaction data");
    let signing_payload: SigningPayload =
        body.payload.as_slice().try_into().context("payload err")?;
    let withdrawal = match signing_payload.body {
        TransactionBody::Withdrawal(withdrawal) => {
            Withdraw::try_from(withdrawal).context("decoding withdrawal")?
        }
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

    tracing::info!("queuing withdrawal transaction");

    state
        .queued_transactions
        .send(TrieStateThreadMsg::Transaction(Signed {
            public_key,
            epoch,
            nonce,
            transaction: Transaction::Withdraw(withdrawal),
        }))
        .await
        .context("sending transaction to trie thread")?;

    Ok(())
}
