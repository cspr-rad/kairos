pub mod error;
pub mod implementations;

#[cfg(feature = "fs")]
use std::path::Path;

use error::CryptoError;

pub trait SignerCore {
    fn sign<T: AsRef<[u8]>>(&self, data: T) -> Result<Vec<u8>, CryptoError>;
    fn verify<T: AsRef<[u8]>, U: AsRef<[u8]>>(
        &self,
        data: T,
        signature_bytes: U,
    ) -> Result<(), CryptoError>;

    #[cfg(feature = "tx")]
    fn verify_tx(&self, tx: kairos_tx::asn::Transaction) -> Result<(), CryptoError>;
    #[cfg(feature = "tx")]
    fn sign_tx_payload(
        &self,
        payload: kairos_tx::asn::SigningPayload,
    ) -> Result<kairos_tx::asn::Transaction, CryptoError>;

    fn to_public_key(&self) -> Result<Vec<u8>, CryptoError>;
}

#[cfg(feature = "fs")]
pub trait SignerFsExtension: SignerCore {
    fn from_private_key_file<P: AsRef<Path>>(file: P) -> Result<Self, CryptoError>
    where
        Self: Sized;
    fn from_public_key<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CryptoError>
    where
        Self: Sized;
}
