pub mod errors;
pub mod routes;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::{routing::post, Router};
use routes::{transfer::Transfer, *};
use tokio::sync::RwLock;

pub use errors::AppErr;

type PublicKey = String;

pub struct BatchState {
    pub balances: HashMap<PublicKey, u64>,
    pub batch_epoch: u64,
    /// The set of transfers that will be batched in the next epoch.
    pub batched_transfers: HashSet<Transfer>,
}
impl BatchState {
    pub fn new() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            balances: HashMap::new(),
            batch_epoch: 0,
            batched_transfers: HashSet::new(),
        }))
    }
}

pub fn app_router(state: Arc<RwLock<BatchState>>) -> Router {
    Router::new()
        .route("/api/v1/mock/deposit", post(deposit))
        .route("/api/v1/mock/withdraw", post(withdraw))
        .route("/api/v1/transfer", post(transfer))
        .with_state(state)
}
