#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::asn::{
    Amount, Deposit, Nonce, PublicKey, SigningPayload, TransactionBody, Transfer, Withdrawal,
};
use crate::error::TxError;

pub fn make_deposit(
    nonce: impl Into<Nonce>,
    amount: impl Into<Amount>,
) -> Result<Vec<u8>, TxError> {
    create_payload(nonce, TransactionBody::Deposit(Deposit::new(amount)))
}

pub fn make_transfer(
    nonce: impl Into<Nonce>,
    recipient: impl Into<PublicKey>,
    amount: impl Into<Amount>,
) -> Result<Vec<u8>, TxError> {
    create_payload(
        nonce,
        TransactionBody::Transfer(Transfer::new(recipient, amount)),
    )
}

pub fn make_withdrawal(
    nonce: impl Into<Nonce>,
    amount: impl Into<Amount>,
) -> Result<Vec<u8>, TxError> {
    create_payload(nonce, TransactionBody::Withdrawal(Withdrawal::new(amount)))
}

// Generic function to create and serialize a payload.
fn create_payload(
    nonce: impl Into<Nonce>,
    body: impl Into<TransactionBody>,
) -> Result<Vec<u8>, TxError> {
    let payload = SigningPayload::new(nonce, body);

    payload.try_into()
}
