use hex::FromHexError;
use thiserror::Error;

use crate::crypto::error::CryptoError;

#[derive(Error, Debug)]
pub enum CliError {
    /// Cryptography error.
    #[error("cryptography error: {error}")]
    CryptoError {
        #[from]
        error: CryptoError,
    },
    // TODO: Add error for "Failed to parse hex string: {}"
    /// Failed to parse hex string.
    #[error("failed to parse hex string: {error}")]
    ParseError {
        #[from]
        error: FromHexError,
    },
}
