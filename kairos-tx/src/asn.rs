use crate::error::TxError;

// Expose types for the public API.
pub use rasn::types::{Integer, OctetString};

use rasn::types::AsnType;
use rasn::{Decode, Encode};

#[derive(AsnType, Encode, Decode, Debug)]
#[rasn(delegate)]
pub struct PublicKey(pub(crate) OctetString);

#[derive(AsnType, Encode, Decode, Debug)]
#[rasn(delegate)]
pub struct Amount(pub(crate) Integer);

#[derive(AsnType, Encode, Decode, Debug)]
#[rasn(delegate)]
pub struct Nonce(pub(crate) Integer);

#[derive(AsnType, Encode, Decode, Debug)]
#[non_exhaustive]
pub struct SigningPayload {
    pub nonce: Nonce,
    pub body: TransactionBody,
}

#[derive(AsnType, Encode, Decode, Debug)]
#[rasn(choice)]
#[non_exhaustive]
pub enum TransactionBody {
    #[rasn(tag(0))]
    Deposit(Deposit),
    #[rasn(tag(1))]
    Transfer(Transfer),
    #[rasn(tag(2))]
    Withdrawal(Withdrawal),
}

#[derive(AsnType, Encode, Decode, Debug)]
#[non_exhaustive]
pub struct Deposit {
    pub amount: Amount,
}

#[derive(AsnType, Encode, Decode, Debug)]
#[non_exhaustive]
pub struct Transfer {
    pub recipient: PublicKey,
    pub amount: Amount,
}

#[derive(AsnType, Encode, Decode, Debug)]
#[non_exhaustive]
pub struct Withdrawal {
    pub amount: Amount,
}

impl TryFrom<&[u8]> for SigningPayload {
    type Error = TxError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        rasn::der::decode(value).map_err(TxError::DecodeError)
    }
}

impl TryFrom<SigningPayload> for Vec<u8> {
    type Error = TxError;

    fn try_from(value: SigningPayload) -> Result<Self, Self::Error> {
        rasn::der::encode(&value).map_err(TxError::EncodeError)
    }
}
