use dotenvy::dotenv;
use kairos_server::config::ServerConfig;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    // loads the environment from the current directories .env file
    // if the .env does not exist in the current directory,
    // we still go ahead and try to obtain a server config from the environment
    let _ = dotenv();
    let config = ServerConfig::from_env()
        .unwrap_or_else(|e| panic!("Failed to parse server config from environment: {}", e));
    kairos_server::run(config).await
}
