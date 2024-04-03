use clap::Parser;
use kairos_test_utils::cctl;
use sd_notify::NotifyState;
use std::path::PathBuf;
use tokio::signal;

#[derive(Parser)]
pub struct Cli {
    #[clap(short, long)]
    pub working_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let _network = cctl::CCTLNetwork::run(cli.working_dir)
        .await
        .expect("An error occured while starting the CCTL network");

    let _ = sd_notify::notify(true, &[NotifyState::Ready]);
    signal::ctrl_c().await?;
    Ok(())
}
