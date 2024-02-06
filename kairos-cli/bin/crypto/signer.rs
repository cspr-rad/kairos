#![allow(unused)]

use super::private_key::CasperPrivateKey;
use super::public_key::CasperPublicKey;
use crate::crypto::error::CryptoError;
use casper_types::{bytesrepr::ToBytes, SecretKey};
use casper_types::{crypto, PublicKey};

pub struct CasperSigner {
    secret_key: CasperPrivateKey,
    public_key: CasperPublicKey,
}

impl CasperSigner {
    pub fn from_key(secret_key: CasperPrivateKey) -> Self {
        // Derive the public key.
        let public_key = CasperPublicKey::from_key(PublicKey::from(&secret_key.0));

        CasperSigner {
            secret_key,
            public_key,
        }
    }

    pub fn from_file(secret_key_path: &str) -> Result<Self, CryptoError> {
        let secret_key =
            SecretKey::from_file(secret_key_path).map_err(|_| CryptoError::FailedToParseKey {})?;

        Ok(Self::from_key(CasperPrivateKey::from_key(secret_key)))
    }

    fn get_public_key(&self) -> CasperPublicKey {
        self.public_key.clone()
    }

    fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let signature = crypto::sign(message, &self.secret_key.0, &self.public_key.0);
        let bytes = signature
            .to_bytes()
            .map_err(|_e| CryptoError::Serialization {})?;

        Ok(bytes)
    }
}
