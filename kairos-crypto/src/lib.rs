pub mod error;
pub mod implementations;

use std::path::Path;

use error::CryptoError;

pub trait CryptoSigner {
    fn from_private_key_file<P: AsRef<Path>>(file: P) -> Result<Self, CryptoError>
    where
        Self: Sized;
    fn from_public_key<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CryptoError>
    where
        Self: Sized;

    fn sign<T: AsRef<[u8]>>(&self, data: T) -> Result<Vec<u8>, CryptoError>;
    fn verify<T: AsRef<[u8]>, U: AsRef<[u8]>>(
        &self,
        data: T,
        signature_bytes: U,
    ) -> Result<(), CryptoError>;

    fn to_public_key(&self) -> Result<Vec<u8>, CryptoError>;
}
