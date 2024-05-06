pub mod config;
pub mod errors;
pub mod routes;
pub mod state;

mod utils;

use std::sync::Arc;

use axum::Router;
use axum_extra::routing::RouterExt;

pub use errors::AppErr;

use crate::config::ServerConfig;
use crate::state::BatchStateManager;

type PublicKey = Vec<u8>;
type Signature = Vec<u8>;

pub fn app_router(state: Arc<state::BatchStateManager>) -> Router {
    Router::new()
        .typed_post(routes::deposit_handler)
        .typed_post(routes::withdraw_handler)
        .typed_post(routes::transfer_handler)
        .with_state(state)
}

pub async fn run(config: ServerConfig) {
    tracing_subscriber::fmt::init();

    let app = app_router(BatchStateManager::new_empty());

    tracing::info!("starting http server on `{}`", config.socket_addr);
    let listener = tokio::net::TcpListener::bind(config.socket_addr)
        .await
        .unwrap();
    tracing::info!("listening on `{}`", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
