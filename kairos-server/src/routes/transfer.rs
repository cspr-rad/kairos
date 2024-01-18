use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{AppErr, BatchState, PublicKey};

#[derive(Serialize, Deserialize)]
pub struct Transfer {
    pub from: PublicKey,
    pub to: PublicKey,
    pub amount: u64,
}

#[derive(Serialize, Deserialize)]
pub struct TransferRequest {
    transfer: Transfer,
    signature: String,
}

pub async fn transfer(
    State(pool): State<Arc<RwLock<BatchState>>>,
    Json(proof_request): Json<TransferRequest>,
) -> Result<(), AppErr> {
    todo!()
}
