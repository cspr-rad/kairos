use assert_cmd::Command;
use reqwest::Url;
use std::path::PathBuf;

// Helper function to get the path to a fixture file
fn fixture_path(relative_path: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.extend(["tests", "fixtures", relative_path].iter());
    path
}

#[tokio::test]
async fn deposit_successful_with_ed25519() {
    let dummy_rpc = Url::parse("http://127.0.0.1:11101").unwrap();
    let kairos = kairos_test_utils::kairos::Kairos::run(dummy_rpc)
        .await
        .unwrap();

    tokio::task::spawn_blocking(move || {
        let secret_key_path = fixture_path("ed25519/secret_key.pem");

        let mut cmd = Command::cargo_bin("kairos-cli").unwrap();
        cmd.arg("--kairos-server-address")
            .arg(kairos.url.as_str())
            .arg("deposit")
            .arg("--amount")
            .arg("123")
            .arg("--private-key")
            .arg(secret_key_path);
        cmd.assert().success().stdout("ok\n");
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
        .arg("--private-key")
        .arg(secret_key_path);
    cmd.assert().success().stdout("ok\n");
}

#[test]
fn withdraw_successful_with_ed25519() {
    let secret_key_path = fixture_path("ed25519/secret_key.pem");

    let mut cmd = Command::cargo_bin("kairos-cli").unwrap();
    cmd.arg("withdraw")
        .arg("--amount")
        .arg("123")
        .arg("--private-key")
        .arg(secret_key_path);
    cmd.assert().success().stdout("ok\n");
}

#[test]
fn deposit_invalid_amount() {
    let secret_key_path = fixture_path("ed25519/secret_key.pem");

    let mut cmd = Command::cargo_bin("kairos-cli").unwrap();
    cmd.arg("deposit")
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
        .arg("--amount")
        .arg("123")
        .arg("--private-key")
        .arg(secret_key_path);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("failed to parse private key"));
}

#[test]
fn deposit_invalid_private_key_content() {
    let secret_key_path = fixture_path("invalid.pem"); // Invalid content

    let mut cmd = Command::cargo_bin("kairos-cli").unwrap();
    cmd.arg("deposit")
        .arg("--amount")
        .arg("123")
        .arg("--private-key")
        .arg(secret_key_path);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("failed to parse private key"));
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
