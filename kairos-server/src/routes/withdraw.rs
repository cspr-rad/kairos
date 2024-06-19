use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::routing::TypedPath;
use tracing::*;

use kairos_circuit_logic::transactions::{KairosTransaction, Signed, Withdraw};
use kairos_tx::asn::{SigningPayload, TransactionBody};

#[cfg(feature = "database")]
use kairos_data::transaction as db;

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
            return Err(AppErr::new(anyhow!("invalid transaction type"))
                .set_status(StatusCode::BAD_REQUEST))
        }
    };
    let public_key = body.public_key;
    let nonce = signing_payload.nonce.try_into().context("decoding nonce")?;

    tracing::info!("queuing withdrawal transaction");

    let withdrawal = KairosTransaction::Withdraw(Signed {
        public_key,
        nonce,
        transaction: withdrawal,
    });
    #[cfg(feature = "database")]
    db::insert(&state.pool, withdrawal.clone()).await?;
    state
        .batch_state_manager
        .enqueue_transaction(withdrawal)
        .await
}
