pub mod config;
pub mod errors;
pub mod routes;
pub mod state;

use axum::Router;
use axum_extra::routing::RouterExt;
use state::LockedBatchState;

pub use errors::AppErr;

type PublicKey = Vec<u8>;
type Signature = Vec<u8>;

pub fn app_router(state: LockedBatchState) -> Router {
    Router::new()
        .typed_post(routes::deposit_handler)
        .typed_post(routes::withdraw_handler)
        .typed_post(routes::transfer_handler)
        .with_state(state)
}
