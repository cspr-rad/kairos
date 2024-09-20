use std::path::PathBuf;

use casper_types::runtime_args;
use cctl::{CCTLNetwork, DeployableContract};

use crate::error::CliError;

pub fn run() -> Result<String, CliError> {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let contract_wasm_path =
            PathBuf::from(env!("PATH_TO_WASM_BINARIES")).join("demo-contract-optimized.wasm");
        let hash_name = "kairos_contract_package_hash";
        let contracts_to_deploy = vec![ DeployableContract {
            hash_name: hash_name.to_string(),
            runtime_args: Some(runtime_args! { "initial_trie_root" => Option::<[u8; 32]>::None }),
            path: contract_wasm_path,
        }];
        println!("Deploying contract...");
        let chainspec_path = PathBuf::from(std::env::var("CCTL_CHAINSPEC").unwrap());
        let config_path = PathBuf::from(std::env::var("CCTL_CONFIG").unwrap());

        let network = CCTLNetwork::run(None, Some(contracts_to_deploy), Some(chainspec_path), Some(config_path))
            .await
            .unwrap();

        println!("Contract deployed successfully!");
        let contract_hash = network.get_contract_hash_for(hash_name);

        let node = network
            .casper_sidecars
            .first()
            .expect("Expected at least one node after successful network run");
        let casper_rpc_url = format!("http://localhost:{}/rpc", node.port.rpc_port);


        println!("Before running the Kairos CLI in another terminal, set the following environment variables:");
        println!("export DEMO_KEYS={}/assets/users", network.working_dir.display());
        println!("export KAIROS_SERVER_DEMO_CONTRACT_HASH={}", contract_hash);
        println!("export KAIROS_SERVER_CASPER_RPC={}", casper_rpc_url);

        let _ = tokio::signal::ctrl_c().await;
    });

    Ok("exiting".to_string())
}
