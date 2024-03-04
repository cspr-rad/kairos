use rasn::error::{DecodeError, EncodeError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TxError {
    /// Errors related to encoding.
    #[error("encode error: {0}")]
    EncodeError(EncodeError),

    /// Errors related to decoding.
    #[error("decode error: {0}")]
    DecodeError(DecodeError),

    /// Constraint violation for a specific field.
    #[error("constraint violated for '{field}'")]
    ConstraintViolation { field: &'static str },
}
