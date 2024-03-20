use axum::extract::Json;
use crate::domain::models::transfers;
use crate::AppState;
use serde::{Deserialize, Serialize};
use axum::extract::State;
use bigdecimal::BigDecimal;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TransferInput {
    sender: String,
    recipient: String,
    amount: BigDecimal,
    signature: Vec<u8>,
}

// When a user commits a transfer it is added to local storage with a processed = false flag
async fn transfer(State(AppState): State<AppState>, Json(TransferInput): Json<TransferInput>) -> Option<transfers::TransferModel> {
    None
}