use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    /// Unable to load a file from the given path.
    #[error("failed to load key from file")]
    KeyLoad,
    /// Failed to parse a public key from a raw data.
    #[error("failed to parse private key")]
    FailedToParseKey,
    /// Invalid public key (hexdigest) or other encoding related error.
    #[error("failed to serialize/deserialize '{context}'")]
    Serialization { context: &'static str },
}
