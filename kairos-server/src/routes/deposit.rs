use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{AppErr, BatchState, PublicKey};

#[derive(Serialize, Deserialize)]
pub struct DepositRequest {
    pub public_key: PublicKey,
    pub amount: u64,
}

pub async fn deposit(
    State(pool): State<Arc<RwLock<BatchState>>>,
    Json(proof_request): Json<DepositRequest>,
) -> Result<(), AppErr> {
    todo!("deposit")
}
