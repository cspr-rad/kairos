use std::net::SocketAddr;

use kairos_server::state::BatchState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let axum_port: u16 = std::env::var("SERVER_PORT").map_or(8000, |x| {
        x.parse().unwrap_or_else(|e| {
            format!("Failed to parse SERVER_PORT: {}", e)
                .parse()
                .unwrap()
        })
    });

    let app = kairos_server::app_router(BatchState::new());

    let axum_addr = SocketAddr::from(([127, 0, 0, 1], axum_port));
    tracing::info!("starting http server");
    let listener = tokio::net::TcpListener::bind(axum_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
