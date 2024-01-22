pub mod errors;
pub mod routes;
pub mod state;

use axum::{routing::post, Router};
use routes::*;
use state::LockedBatchState;

pub use errors::AppErr;

type PublicKey = String;

pub fn app_router(state: LockedBatchState) -> Router {
    Router::new()
        .route("/api/v1/mock/deposit", post(deposit))
        .route("/api/v1/mock/withdraw", post(withdraw))
        .route("/api/v1/transfer", post(transfer))
        .with_state(state)
}
