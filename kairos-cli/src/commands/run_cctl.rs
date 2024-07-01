use std::path::{Path, PathBuf};

use casper_client_types::{runtime_args, RuntimeArgs};
use kairos_test_utils::cctl::{CCTLNetwork, DeployableContract};

use crate::error::CliError;

pub fn run() -> Result<String, CliError> {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let contract_wasm_path =
            PathBuf::from(env!("PATH_TO_WASM_BINARIES")).join("demo-contract-optimized.wasm");
        let hash_name = "kairos_contract_package_hash";
        let contract_to_deploy = DeployableContract {
            hash_name: hash_name.to_string(),
            runtime_args: runtime_args! { "initial_trie_root" => Option::<[u8; 32]>::None },
            path: contract_wasm_path,
        };
        println!("Deploying contract...");
        let chainspec_path = Path::new(env!("CCTL_CHAINSPEC"));
        let config_path = Path::new(env!("CCTL_CONFIG"));

        let network = CCTLNetwork::run(None, Some(contract_to_deploy), Some(chainspec_path), Some(config_path))
            .await
            .unwrap();

        println!("Contract deployed successfully!");
        let contract_hash = network.get_contract_hash_for(hash_name);

        let node = network
            .nodes
            .first()
            .expect("Expected at least one node after successful network run");
        let casper_rpc_url = format!("http://localhost:{}/rpc", node.port.rpc_port);

        println!("You can find demo key pairs in `{:?}`", network.working_dir);

        println!("Before running the Kairos CLI in another terminal, set the following environment variables:");
        println!("export KAIROS_CONTRACT_HASH={}", contract_hash);
        println!("export KAIROS_SERVER_CASPER_RPC={}", casper_rpc_url);

        let _ = tokio::signal::ctrl_c().await;
    });

    Ok("exiting".to_string())
}
