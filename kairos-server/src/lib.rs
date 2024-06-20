pub mod config;
pub mod errors;
pub mod on_deploy;
pub mod routes;
pub mod state;

pub mod l1_sync;
mod utils;

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{self, Duration};

use casper_client::types::DeployHash;
use casper_client_hashing::Digest;
use casper_deploy_notifier::DeployNotifier;

use axum::Router;
use axum_extra::routing::RouterExt;

pub use errors::AppErr;

use crate::config::ServerConfig;
use crate::l1_sync::event_manager::EventManager;
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

pub async fn run(config: ServerConfig) {
    let listener = tokio::net::TcpListener::bind(config.socket_addr)
        .await
        .unwrap_or_else(|err| panic!("Failed to bind to address {}: {}", config.socket_addr, err));
    tracing::info!("listening on `{}`", listener.local_addr().unwrap());

    let state = Arc::new(ServerStateInner {
        batch_state_manager: BatchStateManager::new_empty(),
        server_config: config.clone(),
        known_deposit_deploys: RwLock::new(HashSet::new()),
    });

    let app = app_router(state.clone());

    // deploy notifier
    let (tx, mut rx) = mpsc::channel(100);
    let mut deploy_notifier = DeployNotifier::new(config.casper_sse.as_str());

    tokio::spawn(async move {
        loop {
            if let Err(e) = deploy_notifier.connect().await {
                tracing::error!("Unable to connect: {:?}", e);
                continue;
            }

            if let Err(e) = deploy_notifier.run(tx.clone()).await {
                eprintln!("Error while listening to deployment events: {:?}", e);
            }

            // Connection can sometimes be lost, so we retry after a delay.
            eprintln!("Retrying in 5 seconds...",);
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    let state_x = Arc::clone(&state);
    tokio::spawn(async move {
        let event_manager = EventManager::new(
            &state_x.server_config.casper_rpc,
            &state_x.server_config.kairos_demo_contract_hash,
        )
        .await
        .unwrap();

        let mut interval = time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            tracing::debug!("Triggering periodic L1 sync");
            on_deploy::on_deploy_notification(
                &event_manager.fetcher,
                &event_manager.schemas,
                &state_x.server_config.casper_rpc,
                &state_x.server_config.kairos_demo_contract_hash,
                &state_x.batch_state_manager,
            )
            .await;
        }
    });

    // deploy listener/ callback
    tokio::spawn(async move {
        let event_manager = EventManager::new(
            &state.server_config.casper_rpc,
            &state.server_config.kairos_demo_contract_hash,
        )
        .await
        .unwrap();

        while let Some(notification) = rx.recv().await {
            let deploy_hash = DeployHash::new(Digest::from_hex(notification.deploy_hash).unwrap());
            match state.known_deposit_deploys.write().await.take(&deploy_hash) {
                None => continue,
                Some(_) => {
                    on_deploy::on_deploy_notification(
                        &event_manager.fetcher,
                        &event_manager.schemas,
                        &state.server_config.casper_rpc,
                        &state.server_config.kairos_demo_contract_hash,
                        &state.batch_state_manager,
                    )
                    .await
                }
            }
        }
    });

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
