use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::routing::TypedPath;
use tracing::*;

use kairos_circuit_logic::transactions::{Signed, Transaction, Withdraw};
use kairos_tx::asn::{SigningPayload, TransactionBody};

use crate::routes::PayloadBody;
use crate::state::ServerState;
use crate::AppErr;

#[derive(Debug, TypedPath)]
#[typed_path("/api/v1/withdraw")]
pub struct WithdrawPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn withdraw_handler(
    _: WithdrawPath,
    State(state): State<ServerState>,
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
    let nonce = signing_payload.nonce.try_into().context("decoding nonce")?;

    tracing::info!("queuing withdrawal transaction");

    state
        .batch_state_manager
        .enqueue_transaction(Signed {
            public_key,
            nonce,
            transaction: Transaction::Withdraw(withdrawal),
        })
        .await
}
