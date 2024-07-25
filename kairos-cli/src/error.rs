use crate::client::KairosClientError;
use kairos_crypto::error::CryptoError;

use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    /// Failed to serialize to JSON string.
    #[error("failed to parse hex string: {error}")]
    SerializationError {
        #[from]
        error: serde_json::Error,
    },
    /// Cryptography error.
    #[error("cryptography error: {error}")]
    CryptoError {
        #[from]
        error: CryptoError,
    },
    /// Failed to parse hex string.
    #[error("failed to parse hex string: {error}")]
    ParseError {
        #[from]
        error: FromHexError,
    },
    /// Kairos HTTP client error
    #[error("http client error: {error}")]
    KairosClientError {
        #[from]
        error: KairosClientError,
    },
}
