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
use crate::state::{BatchStateManager, ServerState, ServerStateInner};

/// TODO: support secp256k1
type PublicKey = Vec<u8>;
type Signature = Vec<u8>;

#[cfg(not(feature = "deposit-mock"))]
pub fn app_router(state: ServerState) -> Router {
    Router::new()
        .typed_post(routes::deposit_handler)
        .typed_post(routes::withdraw_handler)
        .typed_post(routes::transfer_handler)
        .with_state(state)
}

#[cfg(feature = "deposit-mock")]
pub fn app_router(state: ServerState) -> Router {
    Router::new()
        .typed_post(routes::deposit_handler)
        .typed_post(routes::withdraw_handler)
        .typed_post(routes::transfer_handler)
        .typed_post(routes::deposit_mock_handler)
        .with_state(state)
}

pub async fn run_l1_sync(server_state: Arc<ServerStateInner>) {
    // Extra check: make sure the default dummy value of contract hash was changed.
    let contract_hash = server_state.server_config.casper_contract_hash.as_str();
    if contract_hash == "0000000000000000000000000000000000000000000000000000000000000000" {
        tracing::warn!(
            "Casper contract hash not configured, L1 synchronization will NOT be enabled."
        );
        return;
    }

    // Initialize L1 synchronizer.
    let l1_sync_service = L1SyncService::new(server_state).await.unwrap_or_else(|e| {
        panic!("Event manager failed to initialize: {}", e);
    });

    // Run periodic synchronization.
    // TODO: Add additional SSE trigger.
    tokio::spawn(async move {
        l1_sync::interval_trigger::run(l1_sync_service.into()).await;
    });
}

pub async fn run(config: ServerConfig) {
    let listener = tokio::net::TcpListener::bind(config.socket_addr)
        .await
        .unwrap_or_else(|err| panic!("Failed to bind to address {}: {}", config.socket_addr, err));
    tracing::info!("listening on `{}`", listener.local_addr().unwrap());

    let state = Arc::new(ServerStateInner {
        batch_state_manager: BatchStateManager::new_empty(&config),
        server_config: config.clone(),
    });

    run_l1_sync(state.clone()).await;

    let app = app_router(state);

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
