use crate::crypto::error::CryptoError;

pub struct CasperPrivateKey(pub casper_types::SecretKey);

impl CasperPrivateKey {
    pub fn from_key(public_key: casper_types::SecretKey) -> Self {
        Self(public_key)
    }

    pub fn from_file(file_path: &str) -> Result<Self, CryptoError> {
        let secret_key = casper_types::SecretKey::from_file(file_path)
            .map_err(|_e| CryptoError::FailedToParseKey {})?;
        Ok(Self(secret_key))
    }
}
