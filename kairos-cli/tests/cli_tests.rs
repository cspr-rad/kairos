use assert_cmd::Command;
use casper_client_types::{runtime_args, RuntimeArgs};
use reqwest::Url;
use std::path::PathBuf;

use casper_client::types::DeployHash;
use casper_client_hashing::Digest;
use kairos_test_utils::{
    cctl::{CCTLNetwork, DeployableContract},
    kairos,
};

// Helper function to get the path to a fixture file
fn fixture_path(relative_path: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.extend(["tests", "fixtures", relative_path].iter());
    path
}

#[tokio::test]
#[cfg_attr(not(feature = "cctl-tests"), ignore)]
async fn deposit_successful_with_ed25519() {
    if std::env::var("RUST_LOG").is_ok() {
        panic!("RUST_LOG is set, which will interfere with the test");
    }

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

    let contract_hash = network.get_contract_hash_for(hash_name);

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

    let kairos = kairos::Kairos::run(&casper_rpc_url, &casper_sse_url, None, None)
        .await
        .unwrap();

    tokio::task::spawn_blocking(move || {
        let depositor_secret_key_path = network
            .working_dir
            .join("assets/users/user-1/secret_key.pem");

        let recipient_pk_path = network
            .working_dir
            .join("assets/users/user-2/public_key_hex");
        let recipient_pk = std::fs::read_to_string(recipient_pk_path).unwrap();

        let mut cmd = Command::cargo_bin("kairos-cli").unwrap();
        cmd.arg("--kairos-server-address")
            .arg(kairos.url.as_str())
            .arg("deposit")
            .arg("--contract-hash")
            .arg(contract_hash.to_string())
            .arg("--amount")
            .arg("123")
            .arg("--private-key")
            .arg(depositor_secret_key_path)
            .arg("--recipient")
            .arg(recipient_pk);

        cmd.assert()
            .success()
            .stdout(predicates::function::function(|stdout: &str| {
                let raw_hash = stdout.trim_end();
                DeployHash::new(
                    Digest::from_hex(raw_hash)
                        .expect("Failed to parse deploy hash after depositing"),
                );
                true
            }));
    })
    .await
    .unwrap();
}

#[test]
fn transfer_successful_with_secp256k1() {
    let secret_key_path = fixture_path("secp256k1/secret_key.pem");
    let recipient = "01a26419a7d82b2263deaedea32d35eee8ae1c850bd477f62a82939f06e80df356"; // Example recipient

    let mut cmd = Command::cargo_bin("kairos-cli").unwrap();
    cmd.arg("transfer")
        .arg("--recipient")
        .arg(recipient)
        .arg("--amount")
        .arg("123")
        .arg("--nonce")
        .arg("0")
        .arg("--private-key")
        .arg(secret_key_path);

    // the transfer command should fail because the server is not running
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("http client error"));
}

#[test]
fn withdraw_successful_with_ed25519() {
    let secret_key_path = fixture_path("ed25519/secret_key.pem");

    let mut cmd = Command::cargo_bin("kairos-cli").unwrap();
    cmd.arg("withdraw")
        .arg("--amount")
        .arg("123")
        .arg("--nonce")
        .arg("0")
        .arg("--private-key")
        .arg(secret_key_path);

    // the transfer command should fail because the server is not running
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("http client error"));
}

#[test]
fn deposit_invalid_amount() {
    let secret_key_path = fixture_path("ed25519/secret_key.pem");

    let mut cmd = Command::cargo_bin("kairos-cli").unwrap();
    cmd.arg("deposit")
        .arg("--contract-hash")
        .arg("000000000000000000000000000000000000")
        .arg("--amount")
        .arg("foo") // Invalid amount
        .arg("--private-key")
        .arg(secret_key_path);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("invalid value"));
}

#[test]
fn deposit_invalid_private_key_path() {
    let secret_key_path = fixture_path("ed25519/non_existing.pem"); // Non-existing path

    let mut cmd = Command::cargo_bin("kairos-cli").unwrap();
    cmd.arg("deposit")
        .arg("--contract-hash")
        .arg("000000000000000000000000000000000000")
        .arg("--amount")
        .arg("123")
        .arg("--recipient")
        .arg("01a26419a7d82b2263deaedea32d35eee8ae1c850bd477f62a82939f06e80df356") // Example recipient
        .arg("--private-key")
        .arg(secret_key_path);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("No such file or directory"));
}

#[test]
fn deposit_invalid_private_key_content() {
    let secret_key_path = fixture_path("invalid.pem"); // Invalid content

    let mut cmd = Command::cargo_bin("kairos-cli").unwrap();
    cmd.arg("deposit")
        .arg("--contract-hash")
        .arg("000000000000000000000000000000000000")
        .arg("--amount")
        .arg("123")
        .arg("--recipient")
        .arg("01a26419a7d82b2263deaedea32d35eee8ae1c850bd477f62a82939f06e80df356") // Example recipient
        .arg("--private-key")
        .arg(secret_key_path);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("cryptography error"));
}

#[test]
fn transfer_invalid_recipient() {
    let secret_key_path = fixture_path("ed25519/secret_key.pem");

    let mut cmd = Command::cargo_bin("kairos-cli").unwrap();
    cmd.arg("transfer")
        .arg("--recipient")
        .arg("foo") // Invalid recipient format
        .arg("--amount")
        .arg("123")
        .arg("--private-key")
        .arg(secret_key_path)
        .arg("--nonce")
        .arg("0");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("failed to parse hex string"));
}

#[test]
fn transfer_valid_recipient() {
    let secret_key_path = fixture_path("ed25519/secret_key.pem");
    let hex_pk = "01e8e1e0b10972e4945d1e493d41be8f39f47bb1299f3248f297d22cbc02010f89";

    let mut cmd = Command::cargo_bin("kairos-cli").unwrap();
    cmd.arg("transfer")
        .arg("--recipient")
        .arg(hex_pk)
        .arg("--amount")
        .arg("123")
        .arg("--private-key")
        .arg(secret_key_path)
        .arg("--nonce")
        .arg("0");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("http client error"));
}
