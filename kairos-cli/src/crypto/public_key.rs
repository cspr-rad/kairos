use crate::crypto::error::CryptoError;
use casper_types::bytesrepr::{FromBytes, ToBytes};

#[derive(Clone)]
pub struct CasperPublicKey(pub casper_types::PublicKey);

impl CasperPublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        let (public_key, _remainder) =
            casper_types::PublicKey::from_bytes(bytes).map_err(|_e| {
                CryptoError::Serialization {
                    context: "public key serialization",
                }
            })?;
        Ok(Self(public_key))
    }

    #[allow(unused)]
    fn to_bytes(&self) -> Result<Vec<u8>, CryptoError> {
        self.0.to_bytes().map_err(|_e| CryptoError::Serialization {
            context: "public key deserialization",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_casper_ed25519_public_key() {
        // This public key has a 01 prefix indicating Ed25519.
        let bytes =
            hex::decode("01c377281132044bd3278b039925eeb3efdb9d99dd5f46d9ec6a764add34581af7")
                .unwrap();
        let result = CasperPublicKey::from_bytes(&bytes);
        assert!(
            result.is_ok(),
            "Ed25519 public key should be parsed correctly"
        );
    }

    #[test]
    fn test_casper_secp256k1_public_key() {
        // This public key has a 02 prefix indicating Secp256k1.
        let bytes =
            hex::decode("0202e99759649fa63a72c685b72e696b30c90f1deabb02d0d9b1de45eb371a73e5bb")
                .unwrap();
        let result = CasperPublicKey::from_bytes(&bytes);
        assert!(
            result.is_ok(),
            "Secp256k1 public key should be parsed correctly"
        );
    }

    #[test]
    fn test_casper_unrecognized_prefix() {
        // Using a 99 prefix which is not recognized.
        let bytes =
            hex::decode("99c377281132044bd3278b039925eeb3efdb9d99dd5f46d9ec6a764add34581af7")
                .unwrap();
        let result = CasperPublicKey::from_bytes(&bytes);
        assert!(
            result.is_err(),
            "Unrecognized prefix should result in an error"
        );
    }
}
