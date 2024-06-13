use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::routing::TypedPath;
use tracing::instrument;

use kairos_circuit_logic::transactions::{KairosTransaction, Signed, Transfer};
use kairos_tx::asn::{SigningPayload, TransactionBody};
use kairos_data::transaction as db;

use crate::{routes::PayloadBody, state::ServerState, AppErr};

#[derive(TypedPath)]
#[typed_path("/api/v1/transfer")]
pub struct TransferPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn transfer_handler(
    _: TransferPath,
    State(state): State<ServerState>,
    Json(body): Json<PayloadBody>,
) -> Result<(), AppErr> {
    tracing::info!("parsing transaction data");
    let signing_payload: SigningPayload =
        body.payload.as_slice().try_into().context("payload err")?;
    let transfer: Transfer = match signing_payload.body {
        TransactionBody::Transfer(transfer) => transfer.try_into().context("decoding transfer")?,
        _ => {
            return Err(AppErr::new(anyhow!("invalid transaction type"))
                .set_status(StatusCode::BAD_REQUEST))
        }
    };
    let public_key = body.public_key;
    let nonce = signing_payload.nonce.try_into().context("decoding nonce")?;

    tracing::info!("TODO: verifying transfer signature");

    tracing::info!("queuing transaction for trie update");

    let transfer = KairosTransaction::Transfer(Signed {
        public_key,
        nonce,
        transaction: transfer,
    });
    let _ = db::insert(state.pool.clone(), transfer.clone()).await;
    state
        .batch_state_manager
        .enqueue_transaction(transfer)
        .await
}
