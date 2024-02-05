use crate::crypto::error::CryptoError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    /// Unable to find argument by name.
    #[error("missing argument '{context}'")]
    MissingArgument { context: &'static str },
    /// Failed to parse amount from string.
    #[error("failed to parse '{context}' as u64")]
    FailedToParseU64 { context: &'static str },
    /// Cryptography error.
    #[error("cryptography error: {error}")]
    CryptoError { error: CryptoError },
}
