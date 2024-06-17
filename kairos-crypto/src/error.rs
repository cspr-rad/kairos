use core::fmt;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[derive(Debug)]
pub enum CryptoError {
    /// Failed to parse a public key from a raw data.
    FailedToParseKey { error: String },
    /// Encoding related error.
    Serialization { context: &'static str },
    /// Invalid public key (hexdigest) or other decoding related error.
    Deserialization { context: &'static str },
    /// Signature verification failure.
    InvalidSignature,
    /// Private key is not provided.
    MissingPrivateKey,
    /// Unable to compute transaction hash - invalid data given.
    #[cfg(feature = "tx")]
    TxHashingError { error: String },
    /// Signing algorithm is not available in `kairos-tx`.
    #[cfg(feature = "tx")]
    InvalidSigningAlgorithm,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::FailedToParseKey { error } => {
                write!(f, "failed to parse private key: {error}")
            }
            CryptoError::Serialization { context } => write!(f, "failed to serialize '{context}'"),
            CryptoError::Deserialization { context } => {
                write!(f, "failed to deserialize '{context}'")
            }
            CryptoError::InvalidSignature => write!(f, "signature verification failed"),
            CryptoError::MissingPrivateKey => write!(f, "private key is not provided"),
            #[cfg(feature = "tx")]
            CryptoError::TxHashingError { error } => {
                write!(f, "unable to hash transaction data: {error}")
            }
            #[cfg(feature = "tx")]
            CryptoError::InvalidSigningAlgorithm => {
                write!(f, "algorithm not available in tx format")
            }
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

    impl Error for CryptoError {}
}

#[cfg(feature = "std")]
impl std::error::Error for CryptoError {}
