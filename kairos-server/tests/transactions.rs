use axum_extra::routing::TypedPath;
use axum_test::{TestServer, TestServerConfig};
use reqwest::Url;
use std::collections::HashSet;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::RwLock;
use tracing_subscriber::{prelude::*, EnvFilter};

use casper_client::{
    types::{DeployBuilder, Timestamp},
    TransferTarget,
};
use casper_client_types::{
    crypto::{PublicKey, SecretKey},
    AsymmetricType, ContractHash,
};
use cctl::CCTLNetwork;
#[cfg(feature = "database")]
use kairos_data::new as new_pool;
use kairos_server::{
    config::{BatchConfig, ServerConfig},
    routes::deposit::DepositPath,
    state::{BatchStateManager, ServerStateInner},
};
#[cfg(feature = "database")]
use kairos_test_utils::postgres::PostgresDB;

#[cfg(feature = "deposit-mock")]
use kairos_server::routes::{
    deposit_mock::MockDepositPath, transfer::TransferPath, withdraw::WithdrawPath, PayloadBody,
};
#[cfg(feature = "deposit-mock")]
use kairos_tx::asn::{SigningPayload, Transfer, Withdrawal};

static TEST_ENVIRONMENT: OnceLock<()> = OnceLock::new();

#[cfg(feature = "deposit-mock")]
async fn new_test_app(#[cfg(feature = "database")] postgres_url: &Url) -> TestServer {
    let dummy_url = Url::parse("http://0.0.0.0:0").unwrap();

    new_test_app_with_casper_node(
        &dummy_url,
        &dummy_url,
        #[cfg(feature = "database")]
        postgres_url,
    )
    .await
}

async fn new_test_app_with_casper_node(
    casper_rpc_url: &Url,
    casper_sse_url: &Url,
    #[cfg(feature = "database")] postgres_url: &Url,
) -> TestServer {
    TEST_ENVIRONMENT.get_or_init(|| {
        tracing_subscriber::registry()
            .with(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info,kairos_server=trace".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    });
    let config = TestServerConfig::builder().mock_transport().build();
    let server_config = ServerConfig {
        secret_key_file: None,
        socket_addr: "0.0.0.0:0".parse().unwrap(),
        casper_rpc: casper_rpc_url.clone(),
        casper_sse: casper_sse_url.clone(),
        // For testing purposes, we set the sync interval to be fast
        casper_sync_interval: Duration::from_secs(5),
        kairos_demo_contract_hash: ContractHash::default(),
        batch_config: BatchConfig {
            max_batch_size: None,
            max_batch_duration: None,
            // dummy proving server will never be called because of max_batch_size and max_batch_duration
            proving_server: Url::parse("http://127.0.0.1:7894").unwrap(),
        },
        #[cfg(feature = "database")]
        db_addr: postgres_url.to_string(),
    };

    let state = Arc::new(ServerStateInner {
        batch_state_manager: BatchStateManager::new_empty(&server_config),
        server_config,
        known_deposit_deploys: RwLock::new(HashSet::new()),
        #[cfg(feature = "database")]
        pool: new_pool(postgres_url.as_ref())
            .await
            .expect("Failed to connect to database"),
    });

    TestServer::new_with_config(kairos_server::app_router(state), config).unwrap()
}

#[tokio::test]
#[cfg_attr(not(feature = "cctl-tests"), ignore)]
async fn test_signed_deploy_is_forwarded_if_sender_in_approvals() {
    let network = CCTLNetwork::run(None, None, None, None).await.unwrap();
    let node = network
        .nodes
        .first()
        .expect("Expected at least one node after successful network run");
    let casper_rpc_url =
        Url::parse(&format!("http://localhost:{}/rpc", node.port.rpc_port)).unwrap();
    let casper_sse_url = Url::parse(&format!(
        "http://localhost:{}/events/main",
        node.port.sse_port
    ))
    .unwrap();

    #[cfg(feature = "database")]
    let postgres = PostgresDB::run(None).unwrap();

    let server = new_test_app_with_casper_node(
        &casper_rpc_url,
        &casper_sse_url,
        #[cfg(feature = "database")]
        &postgres.connection.clone().into(),
    )
    .await;

    let sender_secret_key_file = network
        .working_dir
        .join("assets/users/user-1/secret_key.pem");
    let sender_secret_key = SecretKey::from_file(sender_secret_key_file).unwrap();

    let recipient_public_key_hex = std::fs::read_to_string(
        network
            .working_dir
            .join("assets/users/user-2/public_key_hex"),
    )
    .unwrap();
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
#[cfg(feature = "deposit-mock")]
async fn test_deposit_withdraw() {
    use kairos_circuit_logic::transactions::L1Deposit;

    #[cfg(feature = "database")]
    let postgres = PostgresDB::run(None).unwrap();

    let server = new_test_app(
        #[cfg(feature = "database")]
        &postgres.connection.clone().into(),
    )
    .await;

    let deposit = L1Deposit {
        recipient: "alice_key".into(),
        amount: 100,
    };

    // no arguments
    server
        .post(MockDepositPath.to_uri().path())
        .await
        .assert_status_failure();

    // deposit
    let response = server
        .post(MockDepositPath.to_uri().path())
        .json(&deposit)
        .await;

    println!("{}", &response.text());
    response.assert_status_success();

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
#[cfg(feature = "deposit-mock")]
async fn test_deposit_transfer_withdraw() {
    use kairos_circuit_logic::transactions::L1Deposit;

    #[cfg(feature = "database")]
    let postgres = PostgresDB::run(None).unwrap();

    let server = new_test_app(
        #[cfg(feature = "database")]
        &postgres.connection.clone().into(),
    )
    .await;

    // deposit
    server
        .post(MockDepositPath.to_uri().path())
        .json(&L1Deposit {
            recipient: "alice_key".into(),
            amount: 100,
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
#[cfg(feature = "deposit-mock")]
async fn test_deposit_transfer_to_self_withdraw() {
    use kairos_circuit_logic::transactions::L1Deposit;

    #[cfg(feature = "database")]
    let postgres = PostgresDB::run(None).unwrap();

    let server = new_test_app(
        #[cfg(feature = "database")]
        &postgres.connection.clone().into(),
    )
    .await;

    // deposit
    server
        .post(MockDepositPath.to_uri().path())
        .json(&L1Deposit {
            recipient: "alice_key".into(),
            amount: 1000,
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
        .assert_status_failure();

    // withdraw
    server
        .post(WithdrawPath.to_uri().path())
        .json(&PayloadBody {
            public_key: "alice_key".into(),
            payload: SigningPayload::new(0, Withdrawal::new(1000))
                .try_into()
                .unwrap(),
            signature: vec![],
        })
        .await
        .assert_status_success();
}
