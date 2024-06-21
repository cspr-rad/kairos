use casper_client::types::DeployHash;
use casper_client::types::{DeployBuilder, ExecutableDeployItem, TimeDiff, Timestamp};
use casper_client_types::{crypto::SecretKey, runtime_args, RuntimeArgs};
use reqwest::{blocking, Url};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum KairosClientError {
    ResponseError(String),
    ResponseErrorWithCode(u16, String),
    DecodeError(String),
    CasperClientError(String),
    KairosServerError(String),
}

impl std::error::Error for KairosClientError {}

impl fmt::Display for KairosClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json_string = serde_json::to_string(self).map_err(|_| fmt::Error)?;
        write!(formatter, "{}", json_string)
    }
}
impl From<reqwest::Error> for KairosClientError {
    fn from(error: reqwest::Error) -> Self {
        let error_without_url = error.without_url();
        if error_without_url.is_decode() {
            KairosClientError::DecodeError(error_without_url.to_string())
        } else {
            match error_without_url.status() {
                Option::None => Self::ResponseError(error_without_url.to_string()),
                Option::Some(status_code) => {
                    Self::ResponseErrorWithCode(status_code.as_u16(), error_without_url.to_string())
                }
            }
        }
    }
}

pub fn deposit(
    base_url: &Url,
    depositor_secret_key: &SecretKey,
    amount: u64,
) -> Result<DeployHash, KairosClientError> {
    let deposit_session_wasm_path =
        Path::new(env!("PATH_TO_SESSION_BINARIES")).join("deposit-session-optimized.wasm");
    let deposit_session_wasm_bytes = fs::read(&deposit_session_wasm_path).unwrap_or_else(|err| {
        panic!(
            "Failed to read the deposit session wasm as bytes from file: {:?}.\n{}",
            deposit_session_wasm_path, err
        )
    });
    let deposit_session =
        ExecutableDeployItem::new_module_bytes(deposit_session_wasm_bytes.into(), runtime_args! {});
    let deploy = DeployBuilder::new(
        env!("CASPER_CHAIN_NAME"),
        deposit_session,
        depositor_secret_key,
    )
    .with_standard_payment(amount)
    .with_timestamp(Timestamp::now())
    .with_ttl(TimeDiff::from_millis(60_000)) // 1 min
    .build()
    .map_err(|err| KairosClientError::CasperClientError(err.to_string()))?;

    let client = blocking::Client::new();
    let url = base_url.join("/api/v1/deposit").unwrap();
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&deploy)
        .send()
        .map_err(Into::<KairosClientError>::into)?;
    let status = response.status();
    if !status.is_success() {
        Err(KairosClientError::KairosServerError(status.to_string()))
    } else {
        response
            .json::<DeployHash>()
            .map_err(Into::<KairosClientError>::into)
    }
}
