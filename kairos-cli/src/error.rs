use hex::FromHexError;
use thiserror::Error;

use kairos_crypto::error::CryptoError;

#[derive(Error, Debug)]
pub enum CliError {
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
}
