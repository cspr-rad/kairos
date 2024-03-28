pub mod parsers;
use anyhow::anyhow;
use backoff::{backoff::Constant, future::retry};
use casper_client::{get_node_status, rpcs::results::ReactorState, Error, JsonRpcId, Verbosity};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use tempfile::{tempdir, TempDir};

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
    pub working_dir: TempDir,
    pub assets_dir: PathBuf,
    pub nodes: Vec<CasperNode>,
}

impl CCTLNetwork {
    pub async fn run() -> Result<CCTLNetwork, io::Error> {
        let working_dir = tempdir()?;
        let assets_dir = working_dir.path().join("assets");

        let output = Command::new("cctl-infra-net-setup")
            .env("CCTL_ASSETS", &assets_dir)
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
        retry(
            Constant::new(std::time::Duration::from_millis(100)),
            || async {
                let node_port = nodes.first().unwrap().port.rpc_port;
                get_node_status(
                    JsonRpcId::Number(1),
                    &format!("http://localhost:{}", node_port),
                    Verbosity::High,
                )
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
            },
        )
        .await
        .expect("Waiting for network to pass genesis failed");

        Ok(CCTLNetwork {
            working_dir,
            assets_dir,
            nodes,
        })
    }
}

impl Drop for CCTLNetwork {
    fn drop(&mut self) {
        let output = Command::new("cctl-infra-net-stop")
            .env("CCTL_ASSETS", &self.assets_dir)
            .output()
            .expect("Failed to stop the network");
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_cctl_network_starts_and_terminates() {
        let network = CCTLNetwork::run().await.unwrap();
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
