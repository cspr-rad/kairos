use crate::crypto::error::CryptoError;
use casper_types::bytesrepr::FromBytes;
use casper_types::bytesrepr::ToBytes;
use casper_types::crypto;

#[derive(Clone)]
pub struct CasperPublicKey(pub casper_types::PublicKey);

impl CasperPublicKey {
    pub fn from_hex(hex_str: &str) -> Result<Self, CryptoError> {
        let bytes = hex::decode(hex_str).map_err(|_e| CryptoError::Serialization {})?;
        let (public_key, _) =
            crypto::PublicKey::from_bytes(&bytes).map_err(|_e| CryptoError::Serialization {})?;

        Ok(Self(public_key))
    }

    pub fn from_key(public_key: casper_types::PublicKey) -> Self {
        Self(public_key)
    }

    #[allow(unused)]
    fn get(&self) -> Result<Vec<u8>, CryptoError> {
        self.0
            .to_bytes()
            .map_err(|_e| CryptoError::Serialization {})
    }
}
