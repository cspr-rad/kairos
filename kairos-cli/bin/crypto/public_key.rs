use crate::crypto::error::CryptoError;
use casper_types::bytesrepr::{FromBytes, ToBytes};

#[derive(Clone)]
pub struct CasperPublicKey(pub casper_types::PublicKey);

impl CasperPublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        let (public_key, _remainder) =
            casper_types::PublicKey::from_bytes(bytes).map_err(|_e| {
                CryptoError::Serialization {
                    context: "public key",
                }
            })?;
        Ok(Self(public_key))
    }

    #[allow(unused)]
    fn to_bytes(&self) -> Result<Vec<u8>, CryptoError> {
        self.0.to_bytes().map_err(|_e| CryptoError::Serialization {
            context: "public key",
        })
    }
}
