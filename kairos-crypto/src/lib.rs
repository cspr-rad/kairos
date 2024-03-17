pub mod error;

use std::path::Path;

use error::CryptoError;

pub trait CryptoSigner {
    fn from_private_key_file<P: AsRef<Path>>(file: P) -> Result<Self, CryptoError>
    where
        Self: Sized;
    fn from_public_key(bytes: &[u8]) -> Result<Self, CryptoError>
    where
        Self: Sized;

    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError>;
    fn verify(&self, data: &[u8], signature_bytes: &[u8]) -> Result<(), CryptoError>;

    fn to_public_key(&self) -> Result<Vec<u8>, CryptoError>;
}
