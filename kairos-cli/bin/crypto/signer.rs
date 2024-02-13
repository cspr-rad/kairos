use std::path::Path;

use super::private_key::CasperPrivateKey;
use super::public_key::CasperPublicKey;
use crate::crypto::error::CryptoError;
use casper_types::bytesrepr::ToBytes;
use casper_types::{crypto, PublicKey};

pub struct CasperSigner {
    secret_key: CasperPrivateKey,
    public_key: CasperPublicKey,
}

#[allow(unused)]
impl CasperSigner {
    pub fn from_file<P: AsRef<Path>>(file_path: P) -> Result<Self, CryptoError> {
        let secret_key = CasperPrivateKey::from_file(file_path)?;

        // Derive the public key.
        let public_key = CasperPublicKey::from_key(PublicKey::from(&secret_key.0));

        Ok(CasperSigner {
            secret_key,
            public_key,
        })
    }

    pub fn get_public_key(&self) -> CasperPublicKey {
        self.public_key.clone()
    }

    pub fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let signature = crypto::sign(message, &self.secret_key.0, &self.public_key.0);
        let bytes = signature
            .to_bytes()
            .map_err(|_e| CryptoError::Serialization {
                context: "signature",
            })?;

        Ok(bytes)
    }
}
