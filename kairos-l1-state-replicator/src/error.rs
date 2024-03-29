use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReplicatorError {
    /// CES metadata not found in named keys.
    #[error("metadata key missing: '{context}'")]
    MissingMetadataKey { context: String },

    /// Expected different type of Casper key.
    #[error("key type invalid: '{context}'")]
    InvalidKeyType { context: String },

    #[error("clvalue invalid: {0}")]
    InvalidCLValueType(String),
}
