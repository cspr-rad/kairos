use std::sync::OnceLock;

use axum_extra::routing::TypedPath;
use axum_test::{TestServer, TestServerConfig};
use kairos_server::{
    routes::{deposit::DepositPath, transfer::TransferPath, withdraw::WithdrawPath, PayloadBody},
    state::BatchState,
};
use kairos_tx::helpers::{make_deposit, make_transfer, make_withdrawal};
use tracing_subscriber::{prelude::*, EnvFilter};

static TEST_ENVIRONMENT: OnceLock<()> = OnceLock::new();

fn new_test_app() -> TestServer {
    TEST_ENVIRONMENT.get_or_init(|| {
        tracing_subscriber::registry()
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "trace".into()))
            .with(tracing_subscriber::fmt::layer())
            .init();
    });
    let config = TestServerConfig::builder().mock_transport().build();

    TestServer::new_with_config(kairos_server::app_router(BatchState::new()), config).unwrap()
}

#[tokio::test]
async fn test_deposit_withdraw() {
    let server = new_test_app();

    let nonce: u64 = 1;
    let amount: u64 = 100;
    let deposit = PayloadBody {
        public_key: "alice_key".into(),
        payload: make_deposit(nonce, amount).unwrap(),
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

    let nonce: u64 = 1;
    let amount: u64 = 50;
    let withdrawal = PayloadBody {
        public_key: "alice_key".into(),
        payload: make_withdrawal(nonce, amount).unwrap(),
        signature: vec![],
    };

    // first withdrawal
    server
        .post(WithdrawPath.to_uri().path())
        .json(&withdrawal)
        .await
        .assert_status_success();

    let nonce: u64 = 1;
    let amount: u64 = 51;
    let withdrawal = PayloadBody {
        public_key: "alice_key".into(),
        payload: make_withdrawal(nonce, amount).unwrap(),
        signature: vec![],
    };

    // withdrawal with insufficient funds
    server
        .post(WithdrawPath.to_uri().path())
        .json(&withdrawal)
        .await
        .assert_status_failure();

    let nonce: u64 = 1;
    let amount: u64 = 50;
    let withdrawal = PayloadBody {
        public_key: "alice_key".into(),
        payload: make_withdrawal(nonce, amount).unwrap(),
        signature: vec![],
    };

    // second withdrawal
    server
        .post(WithdrawPath.to_uri().path())
        .json(&withdrawal)
        .await
        .assert_status_success();

    server
        .post(WithdrawPath.to_uri().path())
        .json(&withdrawal)
        .await
        .assert_status_failure();
}

#[tokio::test]
async fn test_deposit_transfer_withdraw() {
    let server = new_test_app();

    let nonce: u64 = 1;
    let amount: u64 = 100;
    let deposit = PayloadBody {
        public_key: "alice_key".into(),
        payload: make_deposit(nonce, amount).unwrap(),
        signature: vec![],
    };

    let nonce: u64 = 1;
    let amount: u64 = 50;
    let recipient: &[u8] = "bob_key".as_bytes();
    let transfer = PayloadBody {
        public_key: "alice_key".into(),
        payload: make_transfer(nonce, recipient, amount).unwrap(),
        signature: vec![],
    };

    let nonce: u64 = 1;
    let amount: u64 = 50;
    let withdrawal = PayloadBody {
        public_key: "bob_key".into(),
        payload: make_withdrawal(nonce, amount).unwrap(),
        signature: vec![],
    };

    // deposit
    server
        .post(DepositPath.to_uri().path())
        .json(&deposit)
        .await
        .assert_status_success();

    // transfer
    server
        .post(TransferPath.to_uri().path())
        .json(&transfer)
        .await
        .assert_status_success();

    // withdraw
    server
        .post(WithdrawPath.to_uri().path())
        .json(&withdrawal)
        .await
        .assert_status_success();
}
