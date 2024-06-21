pub mod parsers;
use anyhow::anyhow;
use backoff::{future::retry, ExponentialBackoff};
use casper_client::{get_node_status, rpcs::results::ReactorState, Error, JsonRpcId, Verbosity};
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

impl CCTLNetwork {
    pub async fn run(
        working_dir: Option<PathBuf>,
        chainspec_path: Option<&Path>,
        config_path: Option<&Path>,
    ) -> Result<CCTLNetwork, io::Error> {
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

        Ok(CCTLNetwork { working_dir, nodes })
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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg_attr(not(feature = "cctl-tests"), ignore)]
    #[tokio::test]
    async fn test_cctl_network_starts_and_terminates() {
        let network = CCTLNetwork::run(Option::None, Option::None, Option::None)
            .await
            .unwrap();
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
}
