use axum_extra::routing::TypedPath;
use casper_client::types::{DeployBuilder, DeployHash, ExecutableDeployItem, TimeDiff, Timestamp};
use casper_client_types::{crypto::SecretKey, runtime_args, ContractHash, RuntimeArgs, U512};
use kairos_server::routes::contract_hash::ContractHashPath;
use kairos_server::routes::deposit::DepositPath;
use kairos_server::routes::get_nonce::GetNoncePath;
use kairos_server::PublicKey;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;

#[cfg(feature = "database")]
use kairos_data::transaction::{TransactionFilter, Transactions};
#[cfg(feature = "database")]
use kairos_server::routes::fetch::QueryTransactionsPath;

// max amount allowed to be used on gas fees
pub const MAX_GAS_FEE_PAYMENT_AMOUNT: u64 = 1_000_000_000_000;

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum KairosClientError {
    ResponseError(String),
    ResponseErrorWithCode(u16, String),
    DecodeError(String),
    CasperClientError(String),
    KairosServerError(u16, String),
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
    contract_hash: &ContractHash,
    amount: impl Into<U512>,
    recipient: casper_client_types::PublicKey,
) -> Result<DeployHash, KairosClientError> {
    let deposit_session_wasm_path =
        Path::new(env!("PATH_TO_SESSION_BINARIES")).join("deposit-session-optimized.wasm");
    let deposit_session_wasm_bytes = fs::read(&deposit_session_wasm_path).unwrap_or_else(|err| {
        panic!(
            "Failed to read the deposit session wasm as bytes from file: {:?}.\n{}",
            deposit_session_wasm_path, err
        )
    });
    let deposit_session = ExecutableDeployItem::new_module_bytes(
        deposit_session_wasm_bytes.into(),
        runtime_args! {
          "demo_contract" => *contract_hash,
          "amount" => amount.into(),
          "recipient" => recipient
        },
    );
    let deploy = DeployBuilder::new(
        env!("CASPER_CHAIN_NAME"),
        deposit_session,
        depositor_secret_key,
    )
    .with_standard_payment(MAX_GAS_FEE_PAYMENT_AMOUNT) // max amount allowed to be used on gas fees
    .with_timestamp(Timestamp::now())
    .with_ttl(TimeDiff::from_millis(60_000)) // 1 min
    .build()
    .map_err(|err| KairosClientError::CasperClientError(err.to_string()))?;

    let response = reqwest::blocking::Client::new()
        .post(base_url.join(DepositPath::PATH).unwrap())
        .header("Content-Type", "application/json")
        .json(&deploy)
        .send()
        .map_err(KairosClientError::from)?
        .error_for_status();

    match response {
        Err(err) => Err(KairosClientError::from(err)),
        Ok(response) => response
            .json::<DeployHash>()
            .map_err(KairosClientError::from),
    }
}

pub fn get_nonce(base_url: &Url, account: &PublicKey) -> Result<u64, KairosClientError> {
    let response = reqwest::blocking::Client::new()
        .post(base_url.join(GetNoncePath::PATH).unwrap())
        .header("Content-Type", "application/json")
        .json(&account)
        .send()
        .map_err(KairosClientError::from)?
        .error_for_status();

    match response {
        Err(err) => Err(KairosClientError::from(err)),
        Ok(response) => response.json::<u64>().map_err(KairosClientError::from),
    }
}

pub fn contract_hash(base_url: &Url) -> Result<ContractHash, KairosClientError> {
    let response = reqwest::blocking::Client::new()
        .get(base_url.join(ContractHashPath::PATH).unwrap())
        .header("Content-Type", "application/json")
        .send()
        .map_err(KairosClientError::from)?;

    let status = response.status();
    if !status.is_success() {
        Err(KairosClientError::KairosServerError(
            status.as_u16(),
            status.to_string(),
        ))
    } else {
        response
            .json::<ContractHash>()
            .map_err(KairosClientError::from)
    }
}

#[cfg(feature = "database")]
pub fn fetch(
    base_url: &Url,
    transaction_filter: &TransactionFilter,
) -> Result<Vec<Transactions>, KairosClientError> {
    let response = reqwest::blocking::Client::new()
        .post(base_url.join(QueryTransactionsPath::PATH).unwrap())
        .header("Content-Type", "application/json")
        .json(&transaction_filter)
        .send()
        .map_err(KairosClientError::from)?
        .error_for_status();

    match response {
        Err(err) => Err(KairosClientError::from(err)),
        Ok(response) => response
            .json::<Vec<Transactions>>()
            .map_err(KairosClientError::from),
    }
}
