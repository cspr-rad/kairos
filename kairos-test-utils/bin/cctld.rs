use clap::Parser;
use kairos_test_utils::cctl;
use sd_notify::NotifyState;
use std::path::PathBuf;
use tokio::signal;

use crate::cctl::DeployableContract;

#[derive(Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub working_dir: Option<PathBuf>,
    #[arg(short, long, num_args(0..))]
    pub deploy_contract: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let deploy_contract =
        cli.deploy_contract.map(|deploy_contracts_arg| {
            match deploy_contracts_arg.split_once(':') {
                Some((hash_name, path)) => DeployableContract {
                    hash_name: hash_name.to_string(),
                    path: PathBuf::from(&path),
                },
                None => panic!("Error parsing the provided deploy contracts argument."),
            }
        });
    let _network = cctl::CCTLNetwork::run(cli.working_dir, deploy_contract)
        .await
        .expect("An error occured while starting the CCTL network");

    let _ = sd_notify::notify(true, &[NotifyState::Ready]);
    signal::ctrl_c().await?;
    Ok(())
}
