use std::u64;

use crate::asn::{
    Amount, Deposit, Epoch, Nonce, PublicKey, SigningPayload, TransactionBody, Transfer, Withdrawal,
};
use crate::asn::{Integer, OctetString};
use crate::error::TxError;

pub fn make_deposit(nonce: u64, epoch: u64, amount: u64) -> Result<Vec<u8>, TxError> {
    create_payload(
        nonce,
        epoch,
        TransactionBody::Deposit(Deposit {
            amount: Amount(Integer::from(amount)),
        }),
    )
}

pub fn make_transfer(
    nonce: u64,
    epoch: u64,
    recipient: &[u8],
    amount: u64,
) -> Result<Vec<u8>, TxError> {
    create_payload(
        nonce,
        epoch,
        TransactionBody::Transfer(Transfer {
            recipient: PublicKey(OctetString::copy_from_slice(recipient)),
            amount: Amount(Integer::from(amount)),
        }),
    )
}

pub fn make_withdrawal(nonce: u64, epoch: u64, amount: u64) -> Result<Vec<u8>, TxError> {
    create_payload(
        nonce,
        epoch,
        TransactionBody::Withdrawal(Withdrawal {
            amount: Amount(Integer::from(amount)),
        }),
    )
}

// Generic function to create and serialize a payload.
fn create_payload(nonce: u64, epoch: u64, body: TransactionBody) -> Result<Vec<u8>, TxError> {
    let payload = SigningPayload {
        nonce: Nonce(Integer::from(nonce)),
        epoch: Epoch(Integer::from(epoch)),
        body,
    };

    payload.try_into()
}
