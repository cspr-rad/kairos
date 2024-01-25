pub mod config;
pub mod errors;
pub mod routes;
pub mod state;

use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use state::LockedBatchState;

pub use errors::AppErr;

type PublicKey = String;

#[derive(TypedPath)]
#[typed_path("/api/v1")]
pub struct ApiV1Path;

pub fn app_router(state: LockedBatchState) -> Router {
    Router::new()
        .typed_post(routes::deposit::handler)
        .typed_post(routes::withdraw::handler)
        .typed_post(routes::transfer::handler)
        .with_state(state)
}
