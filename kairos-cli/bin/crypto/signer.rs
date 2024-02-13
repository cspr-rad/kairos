use std::path::PathBuf;

use super::private_key::CasperPrivateKey;
use super::public_key::CasperPublicKey;
use crate::crypto::error::CryptoError;
use casper_types::bytesrepr::ToBytes;
use casper_types::{crypto, PublicKey, SecretKey};

pub struct CasperSigner {
    secret_key: CasperPrivateKey,
    public_key: CasperPublicKey,
}

#[allow(unused)]
impl CasperSigner {
    fn from_key_raw(secret_key: CasperPrivateKey) -> Self {
        // Derive the public key.
        let public_key = CasperPublicKey::from_key(PublicKey::from(&secret_key.0));

        CasperSigner {
            secret_key,
            public_key,
        }
    }

    pub fn from_file(secret_key_path: &str) -> Result<Self, CryptoError> {
        let secret_key =
            SecretKey::from_file(secret_key_path).map_err(|_e| CryptoError::FailedToParseKey)?;

        Ok(Self::from_key_raw(CasperPrivateKey(secret_key)))
    }

    pub fn from_key_pathbuf(secret_key_path: PathBuf) -> Result<Self, CryptoError> {
        let private_key_path_str: &str = secret_key_path.to_str().ok_or(CryptoError::KeyLoad)?;
        let private_key = CasperPrivateKey::from_file(private_key_path_str)?;

        Ok(Self::from_key_raw(private_key))
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
