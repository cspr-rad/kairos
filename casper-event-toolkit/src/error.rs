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
    InvalidCLValue(String),

    /// Event name not found in loaded schema.
    #[error("event schema missing: {0}")]
    MissingEventSchema(String),

    ///
    #[error("rpc error: {error}")]
    RpcError {
        #[from]
        error: casper_client::Error,
    },

    #[error("parsing error for '{context}'")]
    ParsingError { context: &'static str },

    #[error("deploy error: '{context}'")]
    DeployError { context: &'static str },
}
