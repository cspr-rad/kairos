#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
extern crate alloc;

pub mod asn;
pub mod error;
pub mod helpers;

// Constructor for non-exhaustive transaction struct.
pub fn make_tx(
    public_key: asn::PublicKey,
    payload: asn::SigningPayload,
    algorithm: asn::SigningAlgorithm,
    signature: asn::Signature,
) -> asn::Transaction {
    let transaction = asn::Transaction {
        public_key,
        payload,
        algorithm,
        signature,
    };

    transaction
}
