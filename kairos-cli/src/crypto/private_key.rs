use std::path::Path;

use super::error::CryptoError;

pub struct CasperPrivateKey(pub casper_types::SecretKey);

impl CasperPrivateKey {
    pub fn from_file<P: AsRef<Path>>(file_path: P) -> Result<Self, CryptoError> {
        casper_types::SecretKey::from_file(file_path)
            .map(Self)
            .map_err(|error| error.into())
    }
}
