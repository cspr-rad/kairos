#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
extern crate alloc;

pub mod asn;
pub mod error;
pub mod helpers;

use sha2::Digest;

// Computes the hash for a transaction.
// Hash is obtained from payload by computing sha256 of DER encoded ASN.1 data.
pub fn hash(payload: &asn::SigningPayload) -> Result<[u8; 32], error::TxError> {
    let data = rasn::der::encode(payload).map_err(error::TxError::EncodeError)?;
    let tx_hash: [u8; 32] = sha2::Sha256::digest(data).into();
    Ok(tx_hash)
}

// Constructor for non-exhaustive transaction struct.
pub fn make_tx(
    public_key: asn::PublicKey,
    payload: asn::SigningPayload,
    algorithm: asn::SigningAlgorithm,
    signature: asn::Signature,
) -> asn::Transaction {
    asn::Transaction {
        public_key,
        payload,
        algorithm,
        signature,
    }
}
