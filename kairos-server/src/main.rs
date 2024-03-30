use kairos_server::{config::Settings, state::AppState};

#[tokio::main]
async fn main() {
    let config = Settings::new();
    config.initialize_logger();

    let app = kairos_server::app_router(AppState::new());

    let axum_addr = config.socket_address();
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
