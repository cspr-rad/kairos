use kairos_server::routes::PayloadBody;

use reqwest::{blocking, Url};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum KairosClientError {
    ResponseError(String),
    ResponseErrorWithCode(u16, String),
    DecodeError(String),
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

pub fn submit_transaction_request(
    base_url: &Url,
    deposit_request: &PayloadBody,
) -> Result<(), KairosClientError> {
    let client = blocking::Client::new();
    let url = base_url.join("/api/v1/deposit").unwrap();
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(deposit_request)
        .send()
        .map_err(Into::<KairosClientError>::into)?;
    let status = response.status();
    if !status.is_success() {
        Err(KairosClientError::ResponseError(status.to_string()))
    } else {
        Ok(())
    }
}
