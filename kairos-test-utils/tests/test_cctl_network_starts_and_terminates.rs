use std::path::Path;

use casper_client::{get_node_status, rpcs::results::ReactorState, JsonRpcId, Verbosity};
use kairos_test_utils::cctl::{CCTLNetwork, NodeState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn tracing_init() {
    let _ = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .try_init();
}

#[cfg_attr(not(feature = "cctl-tests"), ignore)]
#[tokio::test]
async fn test_cctl_network_starts_and_terminates() {
    tracing_init();

    let chainspec = Path::new(env!("CCTL_CHAINSPEC"));
    let config = Path::new(env!("CCTL_CONFIG"));

    let network = CCTLNetwork::run(None, None, Some(chainspec), Some(config))
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
