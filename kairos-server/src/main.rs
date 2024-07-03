use dotenvy::dotenv;
use kairos_server::config::ServerConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // loads the environment from the current directories .env file
    // if the .env does not exist in the current directory,
    // we still go ahead and try to obtain a server config from the environment
    let _ = dotenv();
    let config = ServerConfig::from_env()
        .unwrap_or_else(|e| panic!("Failed to parse server config from environment: {}", e));
    kairos_server::run(config).await
}
