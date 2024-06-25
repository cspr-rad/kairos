use assert_cmd::Command;
use reqwest::Url;
use std::path::PathBuf;

use casper_client::types::DeployHash;
use casper_client_hashing::Digest;
use kairos_test_utils::{cctl, kairos};

// Helper function to get the path to a fixture file
fn fixture_path(relative_path: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.extend(["tests", "fixtures", relative_path].iter());
    path
}

#[tokio::test]
#[cfg_attr(not(feature = "cctl-tests"), ignore)]
async fn deposit_successful_with_ed25519() {
    let network = cctl::CCTLNetwork::run(None, None, None, None)
        .await
        .unwrap();
    let node = network
        .nodes
        .first()
        .expect("Expected at least one node after successful network run");
    let casper_rpc_url =
        Url::parse(&format!("http://localhost:{}/rpc", node.port.rpc_port)).unwrap();

    let kairos = kairos::Kairos::run(casper_rpc_url, None).await.unwrap();

    tokio::task::spawn_blocking(move || {
        let depositor_secret_key_path = network
            .working_dir
            .join("assets/users/user-1/secret_key.pem");

        let mut cmd = Command::cargo_bin("kairos-cli").unwrap();
        cmd.arg("--kairos-server-address")
            .arg(kairos.url.as_str())
            .arg("deposit")
            .arg("--contract-hash")
            .arg("--amount")
            .arg("123")
            .arg("--private-key")
            .arg(depositor_secret_key_path);
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
        .arg(secret_key_path);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("failed to parse hex string"));
}
