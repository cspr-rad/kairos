#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::asn::Integer;
use crate::asn::{
    Amount, Deposit, Nonce, PublicKey, SigningPayload, TransactionBody, Transfer, Withdrawal,
};
use crate::error::TxError;

pub fn make_deposit(nonce: u64, amount: impl Into<Amount>) -> Result<Vec<u8>, TxError> {
    create_payload(nonce, TransactionBody::Deposit(Deposit::new(amount)))
}

pub fn make_transfer(
    nonce: u64,
    recipient: impl Into<PublicKey>,
    amount: impl Into<Amount>,
) -> Result<Vec<u8>, TxError> {
    create_payload(
        nonce,
        TransactionBody::Transfer(Transfer::new(recipient, amount)),
    )
}

pub fn make_withdrawal(nonce: u64, amount: impl Into<Amount>) -> Result<Vec<u8>, TxError> {
    create_payload(nonce, TransactionBody::Withdrawal(Withdrawal::new(amount)))
}

// Generic function to create and serialize a payload.
fn create_payload(nonce: u64, body: TransactionBody) -> Result<Vec<u8>, TxError> {
    let payload = SigningPayload {
        nonce: Nonce(Integer::from(nonce)),
        body,
    };

    payload.try_into()
}
