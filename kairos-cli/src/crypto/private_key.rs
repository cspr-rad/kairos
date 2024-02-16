use std::path::Path;

use casper_types::ErrorExt;

pub struct CasperPrivateKey(pub casper_types::SecretKey);

impl CasperPrivateKey {
    pub fn from_file<P: AsRef<Path>>(file_path: P) -> Result<Self, ErrorExt> {
        casper_types::SecretKey::from_file(file_path).map(Self)
    }
}
