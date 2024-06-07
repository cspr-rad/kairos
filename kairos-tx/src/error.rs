use core::fmt;

use rasn::error::{DecodeError, EncodeError};

#[derive(Debug)]
pub enum TxError {
    /// Errors related to encoding.
    EncodeError(EncodeError),

    /// Errors related to decoding.
    DecodeError(DecodeError),

    /// Constraint violation for a specific field.
    ConstraintViolation { field: &'static str },

    /// Signature verification failure.
    InvalidSignature,
}

impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxError::EncodeError(e) => write!(f, "encode error: {e}"),
            TxError::DecodeError(e) => write!(f, "decode error: {e}"),
            TxError::ConstraintViolation { field } => {
                write!(f, "constraint violated for '{field}'")
            }
            TxError::InvalidSignature => write!(f, "signature verification failed"),
        }
    }
}

#[cfg(not(feature = "std"))]
mod error {
    use super::*;
    use core::fmt::{Debug, Display};

    #[allow(dead_code)]
    pub trait Error: Debug + Display {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            None
        }
    }

    impl Error for TxError {}
}

#[cfg(feature = "std")]
impl std::error::Error for TxError {}
