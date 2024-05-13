pub mod config;
pub mod errors;
pub mod routes;
pub mod state;

mod l1_sync;
mod utils;

use std::sync::Arc;

use axum::Router;
use axum_extra::routing::RouterExt;

pub use errors::AppErr;

use crate::config::ServerConfig;
use crate::l1_sync::service::L1SyncService;
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

pub async fn run_l1_sync(config: ServerConfig, batch_service: Arc<BatchStateManager>) {
    // Make sure real contract hash was provided.
    if config.casper_contract_hash
        == "0000000000000000000000000000000000000000000000000000000000000000"
    {
        tracing::warn!(
            "Casper contract hash not configured, L1 synchronization will NOT be enabled."
        );
        return;
    }

    // Run layer 1 synchronization.
    // TODO: Replace interval with SSE trigger.
    let l1_sync_service = Arc::new(L1SyncService::new(batch_service).await);
    tokio::spawn(async move {
        if let Err(e) = l1_sync_service
            .initialize(config.casper_rpc.to_string(), config.casper_contract_hash)
            .await
        {
            panic!("Event manager failed to initialize: {}", e);
        }
        l1_sync::interval_trigger::run(l1_sync_service).await;
    });
}

pub async fn run(config: ServerConfig) {
    let state = BatchStateManager::new_empty();

    run_l1_sync(config.clone(), state.clone()).await;

    let app = app_router(state);

    let listener = tokio::net::TcpListener::bind(config.socket_addr)
        .await
        .unwrap_or_else(|err| panic!("Failed to bind to address {}: {}", config.socket_addr, err));
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
