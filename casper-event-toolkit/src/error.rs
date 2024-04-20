use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolkitError {
    /// CES metadata not found in named keys.
    #[error("metadata key missing: '{context}'")]
    MissingMetadataKey { context: String },

    /// Expected different type of Casper key.
    #[error("key type invalid: '{context}'")]
    InvalidKeyType { context: String },

    #[error("clvalue invalid: {0}")]
    InvalidCLValue(String),

    /// Expected contract type at stored value.
    #[error("stored value type invalid, expected {expected_type}")]
    UnexpectedStoredValueType { expected_type: &'static str },

    /// Unable to get data from RPC.
    #[error("rpc error: {error}")]
    RpcError {
        #[from]
        error: casper_client::Error,
    },

    #[error("parsing error for '{context}'")]
    ParsingError { context: &'static str },

    /// Unable to serialize data into Casper format.
    #[error("serialization error for '{context}'")]
    SerializationError { context: &'static str },

    /// Expected a successful deploy.
    #[error("failed deploy")]
    FailedDeployError,

    /// Event name not found in loaded schema.
    #[error("event '{0}' not found in schema")]
    MissingEventSchema(String),

    /// Event name without required prefix.
    #[error("event prefix not found")]
    MissingEventPrefix,

    /// Unexpected error - should NEVER happen.
    #[error("unexpected error: {context}")]
    UnexpectedError { context: String },
}
