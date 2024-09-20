pub mod config;
pub mod errors;
pub mod routes;
pub mod state;

mod l1_sync;
mod utils;

use axum::Router;
use axum_extra::routing::RouterExt;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use casper_types::contracts::ContractHash;

use crate::config::ServerConfig;
use crate::l1_sync::service::L1SyncService;
use crate::state::{BatchStateManager, ServerState, ServerStateInner};
pub use errors::AppErr;

#[cfg(feature = "database")]
use kairos_data::new as new_pool;

/// TODO: support secp256k1
pub type PublicKey = Vec<u8>;
type Signature = Vec<u8>;

pub fn app_router(state: ServerState) -> Router {
    let mut router = Router::new()
        .typed_post(routes::deposit_handler)
        .typed_post(routes::withdraw_handler)
        .typed_post(routes::transfer_handler)
        .typed_get(routes::get_chain_name_handler)
        .typed_post(routes::get_nonce_handler)
        .typed_get(routes::contract_hash_handler);
    #[cfg(feature = "deposit-mock")]
    {
        router = router.typed_post(routes::deposit_mock_handler)
    }
    #[cfg(feature = "database")]
    {
        router = router.typed_post(routes::query_transactions_handler);
    }
    router.with_state(state)
}

pub async fn run_l1_sync(server_state: Arc<ServerStateInner>) {
    // Extra check: make sure the default dummy value of contract hash was changed.
    let sync_interval = server_state.server_config.casper_sync_interval;
    let contract_hash = server_state.server_config.kairos_demo_contract_hash;
    if contract_hash == ContractHash::default() {
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
        l1_sync_service.run_periodic_sync(sync_interval).await;
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
        known_deposit_deploys: RwLock::new(HashSet::new()),
        #[cfg(feature = "database")]
        pool: new_pool(&config.db_addr)
            .await
            .expect("Failed to connect to database"),
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
