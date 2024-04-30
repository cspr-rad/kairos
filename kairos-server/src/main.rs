use std::net::SocketAddr;

use dotenvy::dotenv;
use kairos_server::{config::ServerConfig, state::BatchStateManager};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let config = ServerConfig::from_env()
        .unwrap_or_else(|e| panic!("Failed to parse server config from environment: {}", e));

    let app = kairos_server::app_router(BatchStateManager::new_empty());

    let axum_addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    tracing::info!("starting http server on `{}`", axum_addr);
    let listener = tokio::net::TcpListener::bind(axum_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
