pub mod config;
pub mod errors;
pub mod routes;
pub mod state;

mod utils;

use axum::Router;
use axum_extra::routing::RouterExt;
use state::AppState;

pub use errors::AppErr;

type PublicKey = Vec<u8>;
type Signature = Vec<u8>;

pub fn app_router(state: AppState) -> Router {
    Router::new()
        .typed_post(routes::deposit_handler)
        .typed_post(routes::withdraw_handler)
        .typed_post(routes::transfer_handler)
        .with_state(state)
}
