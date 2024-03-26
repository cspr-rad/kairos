use lazy_static::lazy_static;

use kairos_server::{config::Settings, state::BatchState};

// Globally accessible config
lazy_static! {
    static ref CONFIG: Settings = Settings::new();
}

#[tokio::main]
async fn main() {
    CONFIG.initialize_logger();

    let app = kairos_server::app_router(BatchState::new());

    let axum_addr = CONFIG.socket_address();

    tracing::info!("Starting HTTP server on `{}`", axum_addr);
    let listener = tokio::net::TcpListener::bind(axum_addr).await.unwrap();
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
