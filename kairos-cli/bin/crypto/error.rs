use casper_types::ErrorExt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    /// Failed to parse a public key from a raw data.
    #[error("failed to parse private key: {error}")]
    FailedToParseKey {
        #[from]
        error: ErrorExt,
    },
    /// Invalid public key (hexdigest) or other encoding related error.
    #[error("failed to serialize/deserialize '{context}'")]
    Serialization { context: &'static str },
}
