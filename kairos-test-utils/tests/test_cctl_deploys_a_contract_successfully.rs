use casper_types::ContractHash;
use hex::FromHex;
use kairos_test_utils::cctl::CCTLNetwork;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use casper_client_types::{runtime_args, RuntimeArgs};
use kairos_test_utils::cctl::DeployableContract;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn tracing_init() {
    let _ = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .try_init();
}

#[cfg_attr(not(feature = "cctl-tests"), ignore)]
#[tokio::test]
async fn test_cctl_deploys_a_contract_successfully() {
    tracing_init();

    let contract_wasm_path =
        PathBuf::from(env!("PATH_TO_WASM_BINARIES")).join("demo-contract-optimized.wasm");
    let hash_name = "kairos_contract_package_hash";
    let contract_to_deploy = DeployableContract {
        hash_name: hash_name.to_string(),
        runtime_args: runtime_args! { "initial_trie_root" => Option::<[u8; 32]>::None },
        path: contract_wasm_path,
    };

    let chainspec = Path::new(env!("CCTL_CHAINSPEC"));
    let config = Path::new(env!("CCTL_CONFIG"));

    let network = CCTLNetwork::run(
        None,
        Some(contract_to_deploy),
        Some(chainspec),
        Some(config),
    )
    .await
    .unwrap();
    let expected_contract_hash_path = network.working_dir.join("contracts").join(hash_name);
    assert!(expected_contract_hash_path.exists());

    let hash_string = fs::read_to_string(expected_contract_hash_path).unwrap();
    let contract_hash_bytes = <[u8; 32]>::from_hex(hash_string).unwrap();
    let contract_hash = ContractHash::new(contract_hash_bytes);
    assert!(contract_hash.to_formatted_string().starts_with("contract-"))
}
