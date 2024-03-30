use std::sync::OnceLock;

use axum_extra::routing::TypedPath;
use axum_test::{TestServer, TestServerConfig};
use kairos_server::{
    routes::{
        deposit::{Deposit, DepositPath},
        transfer::{Transfer, TransferPath},
        withdraw::{WithdrawPath, Withdrawal},
    },
    state::AppState,
};
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

    TestServer::new_with_config(kairos_server::app_router(AppState::new()), config).unwrap()
}

#[tokio::test]
async fn test_deposit_withdraw() {
    let server = new_test_app();

    let deposit = Deposit {
        public_key: "alice_key".into(),
        amount: 100,
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

    let withdrawal = Withdrawal {
        public_key: "alice_key".into(),
        signature: "TODO".into(),
        amount: 50,
    };

    // first withdrawal
    server
        .post(WithdrawPath.to_uri().path())
        .json(&withdrawal)
        .await
        .assert_status_success();

    // withdrawal with insufficient funds
    server
        .post(WithdrawPath.to_uri().path())
        .json(&Withdrawal {
            amount: 51,
            ..withdrawal.clone()
        })
        .await
        .assert_status_failure();

    // second withdrawal
    server
        .post(WithdrawPath.to_uri().path())
        .json(&Withdrawal {
            amount: 50,
            ..withdrawal.clone()
        })
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

    let deposit = Deposit {
        public_key: "alice_key".into(),
        amount: 100,
    };

    let transfer = Transfer {
        from: "alice_key".into(),
        signature: "TODO".into(),
        to: "bob_key".into(),
        amount: 50,
    };

    let withdrawal = Withdrawal {
        public_key: "bob_key".into(),
        signature: "TODO".into(),
        amount: 50,
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
