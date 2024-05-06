use alloc::vec::Vec;

use kairos_tx::{asn, error::TxError};

type PublicKey = Vec<u8>;
// These need to be broken out for use in the zkvm

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Transaction {
    Transfer(Transfer),
    Deposit(Deposit),
    Withdraw(Withdraw),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Signed<T> {
    pub public_key: PublicKey,
    pub nonce: u64,
    pub transaction: T,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Transfer {
    pub recipient: PublicKey,
    pub amount: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Deposit {
    pub amount: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Withdraw {
    pub amount: u64,
}

impl TryFrom<asn::Transfer> for Transfer {
    type Error = TxError;
    fn try_from(transfer: asn::Transfer) -> Result<Self, Self::Error> {
        Ok(Transfer {
            recipient: transfer.recipient.into(),
            amount: transfer.amount.try_into()?,
        })
    }
}

impl TryFrom<asn::Deposit> for Deposit {
    type Error = TxError;
    fn try_from(deposit: asn::Deposit) -> Result<Self, Self::Error> {
        Ok(Deposit {
            amount: deposit.amount.try_into()?,
        })
    }
}

impl TryFrom<asn::Withdrawal> for Withdraw {
    type Error = TxError;
    fn try_from(withdrawal: asn::Withdrawal) -> Result<Self, Self::Error> {
        Ok(Withdraw {
            amount: withdrawal.amount.try_into()?,
        })
    }
}
