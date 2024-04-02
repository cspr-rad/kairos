use crate::error::TxError;

// Expose types for the public API.
pub use rasn::types::{Integer, OctetString};

use num_traits::cast::ToPrimitive;
use rasn::types::AsnType;
use rasn::{Decode, Encode};

#[derive(AsnType, Encode, Decode, Debug)]
#[rasn(delegate)]
pub struct PublicKey(pub(crate) OctetString);

// Converts an ASN.1 decoded public key into raw byte representation.
impl From<PublicKey> for Vec<u8> {
    fn from(value: PublicKey) -> Self {
        value.0.into()
    }
}

#[derive(AsnType, Encode, Decode, Debug)]
#[rasn(delegate)]
pub struct Amount(pub(crate) Integer);

// Attempts to convert an ASN.1 decoded amount (which is represented as a big integer)
// into a `u64`. This conversion can fail if the decoded value is outside the `u64` range,
// thereby enforcing the specified ASN.1 constraints on the value: `INTEGER
// (0..18446744073709551615)`.
impl TryFrom<Amount> for u64 {
    type Error = TxError;

    fn try_from(value: Amount) -> Result<Self, Self::Error> {
        value
            .0
            .to_u64()
            .ok_or(TxError::ConstraintViolation { field: "amount" })
    }
}

#[derive(AsnType, Encode, Decode, Debug)]
#[rasn(delegate)]
pub struct Nonce(pub(crate) Integer);

// Similar to `Amount`, attempts to convert an ASN.1 decoded nonce into a `u64`.
// This is crucial for validating that the nonce adheres to expected constraint:
// `INTEGER (0..18446744073709551615)`.
impl TryFrom<Nonce> for u64 {
    type Error = TxError;

    fn try_from(value: Nonce) -> Result<Self, Self::Error> {
        value
            .0
            .to_u64()
            .ok_or(TxError::ConstraintViolation { field: "nonce" })
    }
}

#[derive(AsnType, Encode, Decode, Debug)]
#[rasn(delegate)]
pub struct Epoch(pub(crate) Integer);

impl TryFrom<Epoch> for u64 {
    type Error = TxError;

    fn try_from(value: Epoch) -> Result<Self, Self::Error> {
        value
            .0
            .to_u64()
            .ok_or(TxError::ConstraintViolation { field: "epoch" })
    }
}

#[derive(AsnType, Encode, Decode, Debug)]
#[non_exhaustive]
pub struct SigningPayload {
    pub nonce: Nonce,
    pub epoch: Epoch,
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

#[cfg(test)]
mod tests {
    use crate::helpers::{make_deposit, make_transfer, make_withdrawal};

    #[test]
    fn test_encode_deposit() {
        const NONCE: u64 = 1;
        const AMOUNT: u64 = 1000;
        let encoded = make_deposit(NONCE, AMOUNT).unwrap();

        assert_eq!(
            encoded,
            vec![
                0b00110000, // T: 0b00 <- universal, 0b1 <- constructed, 0b10000 (16) <- SEQUENCE tag
                0b00001001, // L: 0b0 <- short form, 0b0001001 (9) <- length
                0b00000010, // T: 0b00 <- universal, 0b0 <- primitive, 0b00010 (2) <- INTEGER tag
                0b00000001, // L: 0b0 <- short form, 0b0000001 (1) <- length
                0b00000001, // V: 0b00000001 (1) <- value
                0b10100000, // T: 0b10 <- context-specific, 0b1 <- constructed, 0b00000 (0) <- CHOICE index
                0b00000100, // L: 0b0 <- short form, 0b0000100 (4) <- length
                0b00000010, // T: 0b00 <- universal, 0b0 <- primitive, 0b00010 (2) <- INTEGER tag
                0b00000010, // L: 0b0 <- short form, 0b0000010 (2) <- length
                0b00000011, // V: 512 + 256 +
                0b11101000, //    128 + 64 + 32 + 8 = 1000 <- value
            ]
        );
    }

    #[test]
    fn test_encode_transfer() {
        const NONCE: u64 = 1;
        const RECIPIENT: [u8; 32] = [11; 32];
        const AMOUNT: u64 = 1000;
        let encoded = make_transfer(NONCE, &RECIPIENT, AMOUNT).unwrap();

        assert_eq!(
            encoded,
            vec![
                0x30, 0x2B, // SEQUENCE (43 bytes)
                0x02, 0x01, 0x01, // INTEGER (1 byte), value = 1
                0xA1, 0x26, // CHOICE (38 bytes), index = 1 (transfer body)
                0x04, 0x20, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B,
                0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B,
                0x0B, 0x0B, 0x0B, 0x0B, 0x0B,
                0x0B, // OCTET STRING (32 bytes), value = [11; 32]
                0x02, 0x02, 0x03, 0xE8 // INTEGER (2 bytes), value = 1000
            ]
        );
    }

    #[test]
    fn test_encode_withdrawal() {
        const NONCE: u64 = 1;
        const AMOUNT: u64 = 1000;
        let encoded = make_withdrawal(NONCE, AMOUNT).unwrap();

        assert_eq!(encoded, vec![48, 9, 2, 1, 1, 162, 4, 2, 2, 3, 232]);
    }
}
