pub mod config;
pub mod errors;
pub mod on_deploy;
pub mod routes;
pub mod state;

mod utils;

use backoff::{future::retry, Error, ExponentialBackoff};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use casper_deploy_notifier::DeployNotifier;
use casper_event_toolkit::fetcher::Fetcher;
use casper_event_toolkit::metadata::CesMetadataRef;
use casper_event_toolkit::rpc::client::CasperClient;

use axum::Router;
use axum_extra::routing::RouterExt;

pub use errors::AppErr;

use crate::config::ServerConfig;
use crate::state::{BatchStateManager, ServerState, ServerStateInner};

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
    });
    let app = app_router(Arc::clone(&state));

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

    // deploy listener/ callback
    tokio::spawn(async move {
        let casper_client = CasperClient::new(state.server_config.casper_rpc.as_str());
        let metadata = retry(ExponentialBackoff::default(), || async {
            CesMetadataRef::fetch_metadata(
                &casper_client,
                &state
                    .server_config
                    .kairos_demo_contract_hash
                    .to_formatted_string(),
            )
            .await
            .map_err(Error::transient)
        })
        .await
        .expect("Failed to fetch the demo contracts event metadata");

        let fetcher = Fetcher {
            client: CasperClient::default_mainnet(),
            ces_metadata: metadata,
        };

        let schemas = retry(ExponentialBackoff::default(), || async {
            fetcher.fetch_schema().await.map_err(Error::transient)
        })
        .await
        .expect("Failed to fetch the demo contracts event schema");

        while let Some(notification) = rx.recv().await {
            on_deploy::on_deploy_notification(
                &fetcher,
                &schemas,
                Arc::clone(&state),
                &notification,
            )
            .await;
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
