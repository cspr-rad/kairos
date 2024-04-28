#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::asn::{
    Amount, Deposit, Nonce, PublicKey, SigningPayload, TransactionBody, Transfer, Withdrawal,
};
use crate::asn::{Integer, OctetString};
use crate::error::TxError;

pub fn make_deposit(nonce: u64, amount: impl Into<Amount>) -> Result<Vec<u8>, TxError> {
    create_payload(nonce, TransactionBody::Deposit(Deposit::new(amount)))
}

pub fn make_transfer(nonce: u64, recipient: &[u8], amount: u64) -> Result<Vec<u8>, TxError> {
    create_payload(
        nonce,
        TransactionBody::Transfer(Transfer {
            recipient: PublicKey(OctetString::copy_from_slice(recipient)),
            amount: Amount(Integer::from(amount)),
        }),
    )
}

pub fn make_withdrawal(nonce: u64, amount: u64) -> Result<Vec<u8>, TxError> {
    create_payload(
        nonce,
        TransactionBody::Withdrawal(Withdrawal {
            amount: Amount(Integer::from(amount)),
        }),
    )
}

// Generic function to create and serialize a payload.
fn create_payload(nonce: u64, body: TransactionBody) -> Result<Vec<u8>, TxError> {
    let payload = SigningPayload {
        nonce: Nonce(Integer::from(nonce)),
        body,
    };

    payload.try_into()
}
