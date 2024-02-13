use casper_types::file_utils::read_file;

use crate::crypto::error::CryptoError;

pub struct CasperPrivateKey(pub casper_types::SecretKey);

impl CasperPrivateKey {
    pub fn from_file(file_path: &str) -> Result<Self, CryptoError> {
        let data = read_file(file_path).map_err(|_e| CryptoError::KeyLoad)?;
        let secret_key =
            casper_types::SecretKey::from_pem(data).map_err(|_e| CryptoError::FailedToParseKey)?;
        Ok(Self(secret_key))
    }
}
