mod cli;
mod deployments;
use cli::commander;

// main entry point to CLI
#[tokio::main]
async fn main() {
    commander().await;
}
