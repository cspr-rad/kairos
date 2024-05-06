use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    /// Failed to parse a public key from a raw data.
    #[error("failed to parse private key: {error}")]
    FailedToParseKey { error: String },
    /// Encoding related error.
    #[error("failed to serialize '{context}'")]
    Serialization { context: &'static str },
    /// Invalid public key (hexdigest) or other decoding related error.
    #[error("failed to deserialize '{context}'")]
    Deserialization { context: &'static str },
    /// Signature verification failure.
    #[error("signature verification failed")]
    InvalidSignature,
    /// Private key is not provided.
    #[error("private key is not provided")]
    MissingPrivateKey,

    /// Unable to compute transaction hash - invalid data given.
    #[cfg(feature = "tx")]
    #[error("unable to hash transaction data: {error}")]
    TxHashingError { error: String },
    /// Signing algorithm is not available in `kairos-tx`.
    #[cfg(feature = "tx")]
    #[error("algorithm not available in tx format")]
    InvalidSigningAlgorithm,
}
