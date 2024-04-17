use std::sync::Arc;

use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::routing::TypedPath;
use tracing::instrument;

use kairos_tx::asn::{SigningPayload, TransactionBody};

use crate::{
    routes::PayloadBody,
    state::{
        transactions::{Signed, Transaction, Transfer},
        BatchStateManager, TrieStateThreadMsg,
    },
    AppErr,
};

#[derive(TypedPath)]
#[typed_path("/api/v1/transfer")]
pub struct TransferPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn transfer_handler(
    _: TransferPath,
    State(state): State<Arc<BatchStateManager>>,
    Json(body): Json<PayloadBody>,
) -> Result<(), AppErr> {
    tracing::info!("parsing transaction data");
    let signing_payload: SigningPayload =
        body.payload.as_slice().try_into().context("payload err")?;
    let transfer: Transfer = match signing_payload.body {
        TransactionBody::Transfer(transfer) => transfer.try_into().context("decoding transfer")?,
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

    tracing::info!("TODO: verifying transfer signature");

    tracing::info!("queuing transaction for trie update");

    state
        .queued_transactions
        .send(TrieStateThreadMsg::Transaction(Signed {
            public_key,
            epoch,
            nonce,
            transaction: Transaction::Transfer(transfer),
        }))
        .await
        .context("sending transaction to trie thread")?;

    Ok(())
}
