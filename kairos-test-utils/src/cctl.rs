pub mod parsers;
use anyhow::anyhow;
use backoff::{future::retry, ExponentialBackoff};
use casper_client::{
    get_account, get_deploy, get_node_status, get_state_root_hash, put_deploy, query_global_state,
    rpcs::results::ReactorState,
    types::{DeployBuilder, ExecutableDeployItem, StoredValue, TimeDiff, Timestamp},
    Error, JsonRpcId, Verbosity,
};
use casper_client_types::{ExecutionResult, Key, PublicKey, RuntimeArgs, SecretKey};
use casper_types::ContractHash;
use hex::FromHex;
use rand::Rng;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NodeState {
    Running,
    Stopped,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CasperNodePorts {
    pub consensus_port: u16,
    pub rpc_port: u16,
    pub rest_port: u16,
    pub sse_port: u16,
    pub speculative_exec_port: u16,
}

pub struct CasperNode {
    pub id: u8,
    pub validator_group_id: u8,
    pub state: NodeState,
    pub port: CasperNodePorts,
}

pub struct CCTLNetwork {
    pub working_dir: PathBuf,
    pub nodes: Vec<CasperNode>,
}

pub struct DeployableContract {
    /// This is the named key under which the contract hash is located
    pub hash_name: String,
    pub runtime_args: RuntimeArgs,
    pub path: PathBuf,
}

// max amount allowed to be used on gas fees
pub const MAX_GAS_FEE_PAYMENT_AMOUNT: u64 = 10_000_000_000_000;

impl CCTLNetwork {
    pub async fn run(
        working_dir: Option<PathBuf>,
        contract_to_deploy: Option<DeployableContract>,
        chainspec_path: Option<&Path>,
        config_path: Option<&Path>,
    ) -> anyhow::Result<CCTLNetwork> {
        let working_dir = working_dir
            .map(|dir| {
                std::fs::create_dir_all(&dir)
                    .expect("Failed to create the provided working directory");
                dir
            })
            .unwrap_or(tempdir()?.into_path());
        let assets_dir = working_dir.join("assets");

        let mut setup_command = Command::new("cctl-infra-net-setup");
        setup_command.env("CCTL_ASSETS", &assets_dir);

        if let Some(chainspec_path) = chainspec_path {
            setup_command.arg(format!("chainspec={}", chainspec_path.to_str().unwrap()));
        };

        if let Some(config_path) = config_path {
            setup_command.arg(format!("config={}", config_path.to_str().unwrap()));
        };

        let output = setup_command
            .output()
            .expect("Failed to setup network configuration");
        let output = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        tracing::info!("{}", output);

        let output = Command::new("cctl-infra-net-start")
            .env("CCTL_ASSETS", &assets_dir)
            .output()
            .expect("Failed to start network");
        let output = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        tracing::info!("{}", output);
        let (_, nodes) = parsers::parse_cctl_infra_net_start_lines(output).unwrap();

        let output = Command::new("cctl-infra-node-view-ports")
            .env("CCTL_ASSETS", &assets_dir)
            .output()
            .expect("Failed to get the networks node ports");
        let output = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        tracing::info!("{}", output);
        let (_, node_ports) = parsers::parse_cctl_infra_node_view_port_lines(output).unwrap();

        // Match the started nodes with their respective ports
        let nodes: Vec<CasperNode> = nodes
            .into_iter()
            .map(|(validator_group_id, node_id, state)| {
                if let Some(&(_, port)) = node_ports
                    .iter()
                    .find(|(node_id_ports, _)| *node_id_ports == node_id)
                {
                    CasperNode {
                        validator_group_id,
                        state,
                        id: node_id,
                        port,
                    }
                } else {
                    panic!("Can't find ports for node with id {}", node_id)
                }
            })
            .collect();

        let node_port = nodes.first().unwrap().port.rpc_port;
        let casper_node_rpc_url = format!("http://localhost:{}/rpc", node_port);

        tracing::info!("Waiting for network to pass genesis");
        retry(ExponentialBackoff::default(), || async {
            get_node_status(JsonRpcId::Number(1), &casper_node_rpc_url, Verbosity::Low)
                .await
                .map_err(|err| match &err {
                    Error::ResponseIsHttpError { .. } | Error::FailedToGetResponse { .. } => {
                        backoff::Error::transient(anyhow!(err))
                    }
                    _ => backoff::Error::permanent(anyhow!(err)),
                })
                .map(|success| match success.result.reactor_state {
                    ReactorState::Validate => Ok(()),
                    _ => Err(backoff::Error::transient(anyhow!(
                        "Node didn't reach the VALIDATE state yet"
                    ))),
                })?
        })
        .await
        .expect("Waiting for network to pass genesis failed");

        tracing::info!("Waiting for block 1");
        let output = Command::new("cctl-chain-await-until-block-n")
            .env("CCTL_ASSETS", &assets_dir)
            .arg("height=1")
            .output()
            .expect("Waiting for network to start processing blocks failed");
        let output = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        tracing::info!("{}", output);

        if let Some(contract_to_deploy) = contract_to_deploy {
            let deployer_skey =
                SecretKey::from_file(working_dir.join("assets/users/user-1/secret_key.pem"))?;
            let deployer_pkey =
                PublicKey::from_file(working_dir.join("assets/users/user-1/public_key.pem"))?;

            let (hash_name, contract_hash) = deploy_contract(
                &casper_node_rpc_url,
                &deployer_skey,
                &deployer_pkey,
                &contract_to_deploy,
            )
            .await?;
            let contracts_dir = working_dir.join("contracts");
            fs::create_dir_all(&contracts_dir)?;
            fs::write(
                contracts_dir.join(hash_name),
                // For a ContractHash contract- will always be the prefix
                contract_hash
                    .to_formatted_string()
                    .strip_prefix("contract-")
                    .unwrap(),
            )?
        }
        Ok(CCTLNetwork { working_dir, nodes })
    }
    /// Get the deployed contract hash for a hash_name that was passed to new_contract
    /// https://docs.rs/casper-contract/latest/casper_contract/contract_api/storage/fn.new_contract.html
    pub fn get_contract_hash_for(&self, hash_name: &str) -> ContractHash {
        let contract_hash_path = self.working_dir.join("contracts").join(hash_name);
        let contract_hash_string = fs::read_to_string(contract_hash_path).unwrap();
        let contract_hash_bytes = <[u8; 32]>::from_hex(contract_hash_string).unwrap();
        ContractHash::new(contract_hash_bytes)
    }
}

impl Drop for CCTLNetwork {
    fn drop(&mut self) {
        let output = Command::new("cctl-infra-net-stop")
            .env("CCTL_ASSETS", &self.working_dir.join("assets"))
            .output()
            .expect("Failed to stop the network");
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
    }
}

/// Deploys a contract as the given user for the contract's defined hash name located at the path.
/// The hash name should be equal to the hash name passed to https://docs.rs/casper-contract/latest/casper_contract/contract_api/storage/fn.new_locked_contract.html
async fn deploy_contract(
    casper_node_rpc_url: &str,
    contract_deployer_skey: &SecretKey,
    contract_deployer_pkey: &PublicKey,
    DeployableContract {
        hash_name,
        runtime_args,
        path,
    }: &DeployableContract,
) -> anyhow::Result<(String, casper_client_types::ContractHash)> {
    tracing::info!(
        "Deploying contract {}: {}",
        &hash_name,
        path.to_str().unwrap()
    );

    let contract_bytes = fs::read(path)?;
    let contract =
        ExecutableDeployItem::new_module_bytes(contract_bytes.into(), runtime_args.clone());
    let deploy = DeployBuilder::new(
        // TODO ideally make the chain-name this configurable
        "cspr-dev-cctl",
        contract,
        contract_deployer_skey,
    )
    .with_standard_payment(MAX_GAS_FEE_PAYMENT_AMOUNT) // max amount allowed to be used on gas fees
    .with_timestamp(Timestamp::now())
    .with_ttl(TimeDiff::from_millis(60_000)) // 1 min
    .build()?;

    tracing::info!("Submitting contract deploy");
    let expected_rpc_id = JsonRpcId::Number(rand::thread_rng().gen::<i64>());
    let deploy_hash = put_deploy(
        expected_rpc_id.clone(),
        casper_node_rpc_url,
        Verbosity::High,
        deploy,
    )
    .await
    .map_err(Into::<anyhow::Error>::into)
    .and_then(|response| {
        if response.id == expected_rpc_id {
            Ok(response.result.deploy_hash)
        } else {
            Err(anyhow!("JSON RPC Id missmatch"))
        }
    })?;

    tracing::info!("Waiting for successful contract initialization");
    retry(ExponentialBackoff::default(), || async {
        let expected_rpc_id = JsonRpcId::Number(rand::thread_rng().gen::<i64>());
        let response = get_deploy(
            expected_rpc_id.clone(),
            casper_node_rpc_url,
            Verbosity::High,
            deploy_hash,
            false,
        )
        .await
        .map_err(|err| match &err {
            Error::ResponseIsHttpError { .. } | Error::FailedToGetResponse { .. } => {
                backoff::Error::transient(anyhow!(err))
            }
            _ => backoff::Error::permanent(anyhow!(err)),
        })?;
        if response.id == expected_rpc_id {
            match response.result.execution_results.first() {
                Some(result) => match &result.result {
                    ExecutionResult::Failure { error_message, .. } => {
                        Err(backoff::Error::permanent(anyhow!(error_message.clone())))
                    }
                    ExecutionResult::Success { .. } => Ok(()),
                },
                Option::None => Err(backoff::Error::transient(anyhow!(
                    "No execution results there yet"
                ))),
            }
        } else {
            Err(backoff::Error::permanent(anyhow!("JSON RPC Id missmatch")))
        }
    })
    .await?;
    tracing::info!("Contract was deployed successfully");

    tracing::info!("Fetching deployed contract hash");
    // Query global state
    let expected_rpc_id = JsonRpcId::Number(rand::thread_rng().gen::<i64>());
    let state_root_hash = get_state_root_hash(
        expected_rpc_id.clone(),
        casper_node_rpc_url,
        Verbosity::High,
        Option::None,
    )
    .await
    .map_err(Into::<anyhow::Error>::into)
    .and_then(|response| {
        if response.id == expected_rpc_id {
            response
                .result
                .state_root_hash
                .ok_or(anyhow!("No state root hash present in response"))
        } else {
            Err(anyhow!("JSON RPC Id missmatch"))
        }
    })?;

    let expected_rpc_id = JsonRpcId::Number(rand::thread_rng().gen::<i64>());
    let account = get_account(
        expected_rpc_id.clone(),
        casper_node_rpc_url,
        Verbosity::High,
        Option::None,
        contract_deployer_pkey.clone(),
    )
    .await
    .map_err(Into::<anyhow::Error>::into)
    .and_then(|response| {
        if response.id == expected_rpc_id {
            Ok(response.result.account)
        } else {
            Err(anyhow!("JSON RPC Id missmatch"))
        }
    })?;

    let expected_rpc_id = JsonRpcId::Number(rand::thread_rng().gen::<i64>());
    let account_key = Key::Account(*account.account_hash());
    let contract_hash: casper_client_types::ContractHash = query_global_state(
        expected_rpc_id.clone(),
        casper_node_rpc_url,
        Verbosity::High,
        casper_client::rpcs::GlobalStateIdentifier::StateRootHash(state_root_hash), // fetches recent blocks state root hash
        account_key,
        vec![hash_name.clone()],
    )
    .await
    .map_err(Into::<anyhow::Error>::into)
    .and_then(|response| {
        if response.id == expected_rpc_id {
            match response.result.stored_value {
                StoredValue::ContractPackage(contract_package) => Ok(*contract_package
                    .versions()
                    .next()
                    .expect("Expected at least one contract version")
                    .contract_hash()),
                other => Err(anyhow!(
                    "Unexpected result type, type is not a CLValue: {:?}",
                    other
                )),
            }
        } else {
            Err(anyhow!("JSON RPC Id missmatch"))
        }
    })?;
    tracing::info!(
        "Successfully fetched the contract hash for {}: {}",
        &hash_name,
        &contract_hash
    );
    Ok::<(String, casper_client_types::ContractHash), anyhow::Error>((
        hash_name.clone(),
        contract_hash,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use casper_client_types::runtime_args;
    use hex::FromHex;

    #[cfg_attr(not(feature = "cctl-tests"), ignore)]
    #[tokio::test]
    async fn test_cctl_network_starts_and_terminates() {
        let network = CCTLNetwork::run(None, None, None, None).await.unwrap();
        for node in &network.nodes {
            if node.state == NodeState::Running {
                let node_status = get_node_status(
                    JsonRpcId::Number(1),
                    &format!("http://localhost:{}", node.port.rpc_port),
                    Verbosity::High,
                )
                .await
                .unwrap();
                assert_eq!(node_status.result.reactor_state, ReactorState::Validate);
            }
        }
    }

    #[cfg_attr(not(feature = "cctl-tests"), ignore)]
    #[tokio::test]
    async fn test_cctl_deploys_a_contract_successfully() {
        let contract_wasm_path =
            PathBuf::from(env!("PATH_TO_WASM_BINARIES")).join("demo-contract-optimized.wasm");
        let hash_name = "kairos_contract_package_hash";
        let contract_to_deploy = DeployableContract {
            hash_name: hash_name.to_string(),
            runtime_args: runtime_args! { "initial_trie_root" => Option::<[u8; 32]>::None },
            path: contract_wasm_path,
        };
        let network = CCTLNetwork::run(None, Some(contract_to_deploy), None, None)
            .await
            .unwrap();
        let expected_contract_hash_path = network.working_dir.join("contracts").join(hash_name);
        assert!(expected_contract_hash_path.exists());

        let hash_string = fs::read_to_string(expected_contract_hash_path).unwrap();
        let contract_hash_bytes = <[u8; 32]>::from_hex(hash_string).unwrap();
        let contract_hash = ContractHash::new(contract_hash_bytes);
        assert!(contract_hash.to_formatted_string().starts_with("contract-"))
    }
}
