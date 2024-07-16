use clap::Parser;
use std::process;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() {
    #[cfg(feature = "demo")]
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = kairos_cli::Cli::parse();
    match kairos_cli::run(args) {
        Ok(output) => {
            println!("{}", output)
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    }
}
