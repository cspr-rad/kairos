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
    let app = app_router(BatchStateManager::new_empty());

    let listener = tokio::net::TcpListener::bind(config.socket_addr)
        .await
        .unwrap();
    tracing::info!("listening on `{}`", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {tracing::info!("Received CTRL+C signal, shutting down...")},
        _ = terminate => {tracing::info!("Received shutdown signal, shutting down...")},
    }
}
