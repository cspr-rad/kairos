use std::sync::{Arc, OnceLock};

use axum_extra::routing::TypedPath;
use axum_test::{TestServer, TestServerConfig};
use kairos_server::{
    config::ServerConfig,
    routes::{deposit::DepositPath, transfer::TransferPath, withdraw::WithdrawPath, PayloadBody},
    state::{BatchStateManager, ServerStateInner},
};
use kairos_tx::asn::{SigningPayload, Transfer, Withdrawal};
use tracing_subscriber::{prelude::*, EnvFilter};

static TEST_ENVIRONMENT: OnceLock<()> = OnceLock::new();

fn new_test_app() -> TestServer {
    new_test_app_with_casper_node("0.0.0.0:0")
}

fn new_test_app_with_casper_node(casper_node_url: &str) -> TestServer {
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
            casper_node_url: casper_node_url.to_string(),
        },
    });

    TestServer::new_with_config(kairos_server::app_router(state), config).unwrap()
}
    )
    .unwrap()
}

#[tokio::test]
async fn test_deposit_withdraw() {
    let server = new_test_app();

    let deposit = PayloadBody {
        public_key: "alice_key".into(),
        payload: SigningPayload::new_deposit(100).try_into().unwrap(),
        signature: vec![],
    };

    // no arguments
    server
        .post(DepositPath.to_uri().path())
        .await
        .assert_status_failure();

    // deposit
    server
        .post(DepositPath.to_uri().path())
        .json(&deposit)
        .await
        .assert_status_success();

    // wrong arguments
    server
        .post(WithdrawPath.to_uri().path())
        .json(&deposit)
        .await
        .assert_status_failure();

    // first withdrawal
    server
        .post(WithdrawPath.to_uri().path())
        .json(&PayloadBody {
            public_key: "alice_key".into(),
            payload: SigningPayload::new(0, Withdrawal::new(50))
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
            payload: SigningPayload::new(1, Withdrawal::new(51))
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
            payload: SigningPayload::new(1, Withdrawal::new(50))
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
async fn test_deposit_transfer_withdraw() {
    let server = new_test_app();

    // deposit
    server
        .post(DepositPath.to_uri().path())
        .json(&PayloadBody {
            public_key: "alice_key".into(),
            // deposit's don't have a defined nonce
            payload: SigningPayload::new_deposit(100).try_into().unwrap(),
            signature: vec![],
        })
        .await
        .assert_status_success();

    // transfer
    server
        .post(TransferPath.to_uri().path())
        .json(&PayloadBody {
            public_key: "alice_key".into(),
            payload: SigningPayload::new(0, Transfer::new("bob_key".as_bytes(), 50))
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
            payload: SigningPayload::new(0, Withdrawal::new(50))
                .try_into()
                .unwrap(),
            signature: vec![],
        })
        .await
        .assert_status_success();
}

#[tokio::test]
async fn test_deposit_transfer_to_self_withdraw() {
    let server = new_test_app();

    // deposit
    server
        .post(DepositPath.to_uri().path())
        .json(&PayloadBody {
            public_key: "alice_key".into(),
            // deposit's don't have a defined nonce
            payload: SigningPayload::new_deposit(1000).try_into().unwrap(),
            signature: vec![],
        })
        .await
        .assert_status_success();

    // transfer
    server
        .post(TransferPath.to_uri().path())
        .json(&PayloadBody {
            public_key: "alice_key".into(),
            payload: SigningPayload::new(0, Transfer::new("alice_key".as_bytes(), 1000))
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
            payload: SigningPayload::new(1, Withdrawal::new(1000))
                .try_into()
                .unwrap(),
            signature: vec![],
        })
        .await
        .assert_status_success();
}
