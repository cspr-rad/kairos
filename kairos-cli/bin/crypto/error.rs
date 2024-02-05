use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    /// Failed to parse a public key from a formatted string.
    #[error("failed to parse private key")]
    FailedToParseKey {},
    #[error("failed to serialize signature/key")]
    Serialization {},
}
