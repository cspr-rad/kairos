use std::sync::{Arc, OnceLock};

use axum_extra::routing::TypedPath;
use axum_test::{TestServer, TestServerConfig};
use casper_client::{
    types::{DeployBuilder, Timestamp},
    TransferTarget,
};
use casper_types::{
    crypto::{PublicKey, SecretKey},
    AsymmetricType,
};
use kairos_server::{
    config::ServerConfig,
    routes::{deposit::DepositPath, transfer::TransferPath, withdraw::WithdrawPath, PayloadBody},
    state::{BatchStateManager, ServerStateInner},
};
use kairos_test_utils::cctl::CCTLNetwork;
use kairos_tx::asn::{SigningPayload, Transfer, Withdrawal};
use reqwest::Url;
use tracing_subscriber::{prelude::*, EnvFilter};

static TEST_ENVIRONMENT: OnceLock<()> = OnceLock::new();

fn new_test_app_with_casper_node(casper_node_url: &Url) -> TestServer {
    TEST_ENVIRONMENT.get_or_init(|| {
        tracing_subscriber::registry()
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "trace".into()))
            .with(tracing_subscriber::fmt::layer())
            .init();
    });
    let config = TestServerConfig::builder().mock_transport().build();
    let state = Arc::new(ServerStateInner {
        batch_state_manager: BatchStateManager::new_empty(),
        server_config: ServerConfig {
            socket_addr: "0.0.0.0:0".parse().unwrap(),
            casper_rpc: casper_node_url.clone(),
        },
    });

    TestServer::new_with_config(kairos_server::app_router(state), config).unwrap()
}

#[tokio::test]
#[cfg_attr(not(feature = "cctl-tests"), ignore)]
async fn test_signed_deploy_is_forwarded_if_sender_in_approvals() {
    let network = CCTLNetwork::run().await.unwrap();
    let node = network
        .nodes
        .first()
        .expect("Expected at least one node after successful network run");
    let casper_node_url =
        Url::parse(&format!("http://localhost:{}/rpc", node.port.rpc_port)).unwrap();

    let server = new_test_app_with_casper_node(&casper_node_url);

    let sender_secret_key_file = network.assets_dir.join("users/user-1/secret_key.pem");
    let sender_secret_key = SecretKey::from_file(sender_secret_key_file).unwrap();

    let recipient_public_key_hex =
        std::fs::read_to_string(network.assets_dir.join("users/user-2/public_key_hex")).unwrap();
    let recipient = PublicKey::from_hex(recipient_public_key_hex).unwrap();

    // DeployBuilder::build, calls Deploy::new, which calls Deploy::sign
    let deploy = DeployBuilder::new_transfer(
        "cspr-dev-cctl",
        2_500_000_000u64,
        // Option::None use the accounts main purse
        Option::None,
        TransferTarget::PublicKey(recipient),
        Option::None,
        &sender_secret_key,
    )
    .with_timestamp(Timestamp::now())
    .with_standard_payment(2_500_000_000u64)
    .build()
    .unwrap();

    server
        .post(DepositPath.to_uri().path())
        .json(&deploy)
        .await
        .assert_status_success();
}

#[tokio::test]
#[cfg_attr(not(feature = "cctl-tests"), ignore)]
async fn test_deposit_withdraw() {
    let network = CCTLNetwork::run().await.unwrap();
    let node = network
        .nodes
        .first()
        .expect("Expected at least one node after successful network run");
    let casper_node_url =
        Url::parse(&format!("http://localhost:{}/rpc", node.port.rpc_port)).unwrap();

    let server = new_test_app_with_casper_node(&casper_node_url);

    // DeployBuilder::build, calls Deploy::new, which calls Deploy::sign
    let deposit_deploy = DeployBuilder::new_transfer(
        "cspr-dev-cctl",
        5_000_000_000u64,
        // Option::None use the accounts main purse
        Option::None,
        TransferTarget::PublicKey(PublicKey::generate_ed25519().unwrap()),
        Option::None,
        &SecretKey::generate_ed25519().unwrap(),
    )
    .with_timestamp(Timestamp::now())
    .with_standard_payment(5_000_000_000u64)
    .build()
    .unwrap();

    // no arguments
    server
        .post(DepositPath.to_uri().path())
        .await
        .assert_status_failure();

    // deposit
    server
        .post(DepositPath.to_uri().path())
        .json(&deposit_deploy)
        .await
        .assert_status_success();

    // wrong arguments
    server
        .post(WithdrawPath.to_uri().path())
        .json(&deposit_deploy)
        .await
        .assert_status_failure();

    // first withdrawal
    server
        .post(WithdrawPath.to_uri().path())
        .json(&PayloadBody {
            public_key: "alice_key".into(),
            payload: SigningPayload::new(0, Withdrawal::new(2_500_000_000u64))
                .try_into()
                .unwrap(),
            signature: vec![],
        })
        .await
        .assert_status_success();

    // withdrawal with insufficient funds
    server
        .post(WithdrawPath.to_uri().path())
        .json(&PayloadBody {
            public_key: "alice_key".into(),
            payload: SigningPayload::new(1, Withdrawal::new(2_600_000_000u64))
                .der_encode()
                .unwrap(),
            signature: vec![],
        })
        .await
        .assert_status_failure();

    // second withdrawal
    server
        .post(WithdrawPath.to_uri().path())
        .json(&PayloadBody {
            public_key: "alice_key".into(),
            payload: SigningPayload::new(1, Withdrawal::new(2_500_000_000u64))
                .try_into()
                .unwrap(),
            signature: vec![],
        })
        .await
        .assert_status_success();

    server
        .post(WithdrawPath.to_uri().path())
        .json(&PayloadBody {
            public_key: "alice_key".into(),
            payload: SigningPayload::new(2, Withdrawal::new(50))
                .try_into()
                .unwrap(),
            signature: vec![],
        })
        .await
        .assert_status_failure();
}

#[tokio::test]
#[cfg_attr(not(feature = "cctl-tests"), ignore)]
async fn test_deposit_transfer_withdraw() {
    let network = CCTLNetwork::run().await.unwrap();
    let node = network
        .nodes
        .first()
        .expect("Expected at least one node after successful network run");
    let casper_node_url =
        Url::parse(&format!("http://localhost:{}/rpc", node.port.rpc_port)).unwrap();

    let server = new_test_app_with_casper_node(&casper_node_url);

    // DeployBuilder::build, calls Deploy::new, which calls Deploy::sign
    let deposit_deploy = DeployBuilder::new_transfer(
        "cspr-dev-cctl",
        5_000_000_000u64,
        // Option::None use the accounts main purse
        Option::None,
        TransferTarget::PublicKey(PublicKey::generate_ed25519().unwrap()),
        Option::None,
        &SecretKey::generate_ed25519().unwrap(),
    )
    .with_timestamp(Timestamp::now())
    .with_standard_payment(5_000_000_000u64)
    .build()
    .unwrap();

    // deposit
    server
        .post(DepositPath.to_uri().path())
        .json(&deposit_deploy)
        .await
        .assert_status_success();

    // transfer
    server
        .post(TransferPath.to_uri().path())
        .json(&PayloadBody {
            public_key: "alice_key".into(),
            payload: SigningPayload::new(0, Transfer::new("bob_key".as_bytes(), 2_500_000_000u64))
                .try_into()
                .unwrap(),
            signature: vec![],
        })
        .await
        .assert_status_success();

    // withdraw
    server
        .post(WithdrawPath.to_uri().path())
        .json(&PayloadBody {
            public_key: "bob_key".into(),
            payload: SigningPayload::new(0, Withdrawal::new(2_500_000_000u64))
                .try_into()
                .unwrap(),
            signature: vec![],
        })
        .await
        .assert_status_success();
}

#[tokio::test]
#[cfg_attr(not(feature = "cctl-tests"), ignore)]
async fn test_deposit_transfer_to_self_withdraw() {
    let network = CCTLNetwork::run().await.unwrap();
    let node = network
        .nodes
        .first()
        .expect("Expected at least one node after successful network run");
    let casper_node_url =
        Url::parse(&format!("http://localhost:{}/rpc", node.port.rpc_port)).unwrap();

    let server = new_test_app_with_casper_node(&casper_node_url);

    // DeployBuilder::build, calls Deploy::new, which calls Deploy::sign
    let deposit_deploy = DeployBuilder::new_transfer(
        "cspr-dev-cctl",
        2_500_000_000u64,
        // Option::None use the accounts main purse
        Option::None,
        TransferTarget::PublicKey(PublicKey::generate_ed25519().unwrap()),
        Option::None,
        &SecretKey::generate_ed25519().unwrap(),
    )
    .with_timestamp(Timestamp::now())
    .with_standard_payment(2_500_000_000u64)
    .build()
    .unwrap();

    // deposit
    server
        .post(DepositPath.to_uri().path())
        .json(&deposit_deploy)
        .await
        .assert_status_success();

    // transfer
    server
        .post(TransferPath.to_uri().path())
        .json(&PayloadBody {
            public_key: "alice_key".into(),
            payload: SigningPayload::new(
                0,
                Transfer::new("alice_key".as_bytes(), 2_500_000_000u64),
            )
            .try_into()
            .unwrap(),
            signature: vec![],
        })
        .await
        .assert_status_success();

    // withdraw
    server
        .post(WithdrawPath.to_uri().path())
        .json(&PayloadBody {
            public_key: "alice_key".into(),
            payload: SigningPayload::new(1, Withdrawal::new(2_500_000_000u64))
                .try_into()
                .unwrap(),
            signature: vec![],
        })
        .await
        .assert_status_success();
}
