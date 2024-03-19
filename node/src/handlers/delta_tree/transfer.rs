use axum::extract::Json;
use crate::domain::models::transfers;
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TransferInput {
    sender: String,
    recipient: String,
    amount: BigDecimal,
    signature: Vec<u8>,
}

// When a user commits a transfer it is added to local storage with a processed = false flag
async fn transfer(Json(TransferInput): Json<TransferInput>) -> transfers::TransferModel {
    
}