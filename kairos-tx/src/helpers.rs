#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::asn::{
    Amount, Deposit, Nonce, PublicKey, SigningPayload, TransactionBody, Transfer, Withdrawal,
};

pub fn make_deposit(nonce: impl Into<Nonce>, amount: impl Into<Amount>) -> SigningPayload {
    SigningPayload::new(nonce, TransactionBody::Deposit(Deposit::new(amount)))
}

pub fn make_transfer(
    nonce: impl Into<Nonce>,
    recipient: impl Into<PublicKey>,
    amount: impl Into<Amount>,
) -> SigningPayload {
    SigningPayload::new(
        nonce,
        TransactionBody::Transfer(Transfer::new(recipient, amount)),
    )
}

pub fn make_withdrawal(nonce: impl Into<Nonce>, amount: impl Into<Amount>) -> SigningPayload {
    SigningPayload::new(nonce, TransactionBody::Withdrawal(Withdrawal::new(amount)))
}
