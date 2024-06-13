use casper_types::bytesrepr::{FromBytes, ToBytes};
use casper_types::{crypto, PublicKey, SecretKey, Signature};

#[cfg(feature = "fs")]
use std::path::Path;

extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

use crate::CryptoError;
use crate::SignerCore;

pub struct Signer {
    private_key: Option<SecretKey>,
    public_key: PublicKey,
}

impl SignerCore for Signer {
    fn sign<T: AsRef<[u8]>>(&self, data: T) -> Result<Vec<u8>, CryptoError> {
        let private_key = self
            .private_key
            .as_ref()
            .ok_or(CryptoError::MissingPrivateKey)?;
        let signature = crypto::sign(data, private_key, &self.public_key);
        let signature_bytes = signature
            .to_bytes()
            .map_err(|_e| CryptoError::Serialization {
                context: "signature",
            })?;

        Ok(signature_bytes)
    }

    fn verify<T: AsRef<[u8]>, U: AsRef<[u8]>>(
        &self,
        data: T,
        signature_bytes: U,
    ) -> Result<(), CryptoError> {
        let (signature, _remainder) =
            Signature::from_bytes(signature_bytes.as_ref()).map_err(|_e| {
                CryptoError::Deserialization {
                    context: "signature",
                }
            })?;
        crypto::verify(data, &signature, &self.public_key)
            .map_err(|_e| CryptoError::InvalidSignature)?;

        Ok(())
    }

    fn to_public_key(&self) -> Result<Vec<u8>, CryptoError> {
        let public_key =
            self.public_key
                .clone()
                .into_bytes()
                .map_err(|_e| CryptoError::Serialization {
                    context: "public_key",
                })?;

        Ok(public_key)
    }
}

#[cfg(feature = "fs")]
impl crate::SignerFsExtension for Signer {
    fn from_private_key_file<P: AsRef<Path>>(file: P) -> Result<Self, CryptoError>
    where
        Self: Sized,
    {
        let private_key =
            SecretKey::from_file(file).map_err(|e| CryptoError::FailedToParseKey {
                error: e.to_string(),
            })?;
        let public_key = PublicKey::from(&private_key);

        Ok(Self {
            private_key: Some(private_key),
            public_key,
        })
    }

    fn from_public_key<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CryptoError>
    where
        Self: Sized,
    {
        let (public_key, _remainder) = casper_types::PublicKey::from_bytes(bytes.as_ref())
            .map_err(|_e| CryptoError::Deserialization {
                context: "public key",
            })?;

        Ok(Self {
            private_key: None,
            public_key,
        })
    }
}

#[cfg(feature = "tx")]
impl crate::SignerTxExtension for Signer {
    fn verify_tx(&self, tx: kairos_tx::asn::Transaction) -> Result<(), CryptoError> {
        let tx_hash = tx.payload.hash().map_err(|e| CryptoError::TxHashingError {
            error: e.to_string(),
        })?;
        let signature: Vec<u8> = tx.signature.into();
        self.verify(tx_hash, signature)?;

        Ok(())
    }

    fn sign_tx_payload(
        &self,
        payload: kairos_tx::asn::SigningPayload,
    ) -> Result<kairos_tx::asn::Transaction, CryptoError> {
        // Compute payload signature.
        let tx_hash = payload.hash().map_err(|e| CryptoError::TxHashingError {
            error: e.to_string(),
        })?;
        let signature = self.sign(tx_hash)?;

        // Prepare public key.
        let public_key = self.to_public_key()?;

        // Prepare algorithm.
        let algorithm = match self.public_key {
            PublicKey::Ed25519(_) => Ok(kairos_tx::asn::SigningAlgorithm::CasperEd25519),
            PublicKey::Secp256k1(_) => Ok(kairos_tx::asn::SigningAlgorithm::CasperSecp256k1),
            _ => Err(CryptoError::InvalidSigningAlgorithm),
        }?;

        // Build full transaction.
        let tx = kairos_tx::asn::Transaction::new(
            public_key,
            payload,
            &tx_hash,
            algorithm,
            signature.into(),
        );

        Ok(tx)
    }
}

#[cfg(test)]
#[cfg(feature = "fs")]
mod tests {
    use super::*;
    use crate::SignerFsExtension;

    #[test]
    fn test_casper_ed25519_public_key() {
        // This public key has a 01 prefix indicating Ed25519.

        let bytes =
            hex::decode("01c377281132044bd3278b039925eeb3efdb9d99dd5f46d9ec6a764add34581af7")
                .unwrap();
        let result = Signer::from_public_key(bytes);
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
        let result = Signer::from_public_key(bytes);
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
        let result = Signer::from_public_key(bytes);
        assert!(
            result.is_err(),
            "Unrecognized prefix should result in an error"
        );
    }
}
