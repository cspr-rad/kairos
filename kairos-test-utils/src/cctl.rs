pub mod parsers;
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
            .expect("failed to execute setup network config");
        let output = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        tracing::info!("{}", output);

        let output = Command::new("cctl-infra-net-start")
            .env("CCTL_ASSETS", &assets_dir)
            .output()
            .expect("failed to start network");
        let output = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        tracing::info!("{}", output);
        let (_, nodes) = parsers::parse_cctl_infra_net_start_lines(output).unwrap();

        let output = Command::new("cctl-infra-node-view-ports")
            .env("CCTL_ASSETS", &assets_dir)
            .output()
            .expect("failed to get node ports");
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

        tracing::info!("Waiting for network to start processing blocks");
        std::thread::sleep(std::time::Duration::from_secs(5));
        let output = Command::new("cctl-chain-await-until-block-n")
            .env("CCTL_ASSETS", &assets_dir)
            .arg("height=0")
            .output()
            .expect("Waiting for network to start processing blocks failed");
        let output = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        tracing::info!("{}", output);

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
            .expect("failed to execute setup network config");
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use casper_client::{get_state_root_hash, JsonRpcId, Verbosity};
    #[tokio::test]
    async fn test_cctl_network_starts_and_terminates() {
        let network = CCTLNetwork::run().await.unwrap();
        let node_port = network.nodes.first().unwrap().port.rpc_port;
        get_state_root_hash(
            JsonRpcId::Number(1),
            &format!("http://localhost:{}", node_port),
            Verbosity::High,
            Option::None,
        )
        .await
        .unwrap();
    }
}
