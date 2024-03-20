use axum::extract::{rejection::FailedToDeserializeForm, Json};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::domain::models::transfers;
use crate::AppState;
use serde::{Deserialize, Serialize};
use axum::extract::State;
use chrono::{Utc, NaiveDateTime};
use bigdecimal::BigDecimal;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TransferInput {
    sender: String,
    recipient: String,
    amount: BigDecimal,
    signature: Vec<u8>,
}

// When a user commits a transfer it is added to local storage with a processed = false flag
async fn transfer(State(AppState): State<AppState>, Json(TransferInput): Json<TransferInput>) -> impl IntoResponse {
    let state = State(AppState);
    let transfer = transfers::TransferModel {
        sender: TransferInput.sender,
        recipient: TransferInput.recipient,
        amount: TransferInput.amount,
        timestamp: Utc::now().naive_utc(),
        sig: TransferInput.signature,
        processed: false,
        nonce: 0.into(),
    };
    transfers::insert(state.pool.clone(), transfer.into()).await; // handle errors here
    (StatusCode::OK, "Transfer submitted successfully!").into_response()
}