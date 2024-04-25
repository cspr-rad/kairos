#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::error::TxError;

// Expose types for the public API.
pub use rasn::types::{Integer, OctetString};

use num_traits::cast::ToPrimitive;
use rasn::types::AsnType;
use rasn::{Decode, Encode};

#[derive(AsnType, Encode, Decode, Debug)]
#[rasn(delegate)]
pub struct PublicKey(pub(crate) OctetString);

impl From<&[u8]> for PublicKey {
    fn from(value: &[u8]) -> Self {
        PublicKey(OctetString::copy_from_slice(value))
    }
}

// Converts an ASN.1 decoded public key into raw byte representation.
impl From<PublicKey> for Vec<u8> {
    fn from(value: PublicKey) -> Self {
        value.0.into()
    }
}

impl<const N: usize> From<[u8; N]> for PublicKey {
    fn from(value: [u8; N]) -> Self {
        PublicKey(OctetString::copy_from_slice(&value))
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

impl From<u64> for Amount {
    fn from(value: u64) -> Self {
        Amount(Integer::from(value))
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

impl From<u64> for Nonce {
    fn from(value: u64) -> Self {
        Nonce(Integer::from(value))
    }
}

#[derive(AsnType, Encode, Decode, Debug)]
#[non_exhaustive]
pub struct SigningPayload {
    pub nonce: Nonce,
    pub body: TransactionBody,
}

impl SigningPayload {
    pub fn new(nonce: impl Into<Nonce>, body: impl Into<TransactionBody>) -> Self {
        Self {
            nonce: nonce.into(),
            body: body.into(),
        }
    }

    pub fn new_deposit(amount: impl Into<Amount>) -> Self {
        Self {
            // deposits have no meaningful nonce
            nonce: 0.into(),
            body: TransactionBody::Deposit(Deposit::new(amount)),
        }
    }

    pub fn new_transfer(
        nonce: impl Into<Nonce>,
        recipient: impl Into<PublicKey>,
        amount: impl Into<Amount>,
    ) -> Self {
        Self {
            nonce: nonce.into(),
            body: TransactionBody::Transfer(Transfer::new(recipient, amount)),
        }
    }

    pub fn new_withdrawal(nonce: impl Into<Nonce>, amount: impl Into<Amount>) -> Self {
        Self {
            nonce: nonce.into(),
            body: TransactionBody::Withdrawal(Withdrawal::new(amount)),
        }
    }

    pub fn der_encode(&self) -> Result<Vec<u8>, TxError> {
        rasn::der::encode(self).map_err(TxError::EncodeError)
    }

    pub fn der_decode(value: impl AsRef<[u8]>) -> Result<Self, TxError> {
        rasn::der::decode(value.as_ref()).map_err(TxError::DecodeError)
    }
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

impl From<Deposit> for TransactionBody {
    fn from(value: Deposit) -> Self {
        TransactionBody::Deposit(value)
    }
}

impl From<Transfer> for TransactionBody {
    fn from(value: Transfer) -> Self {
        TransactionBody::Transfer(value)
    }
}

impl From<Withdrawal> for TransactionBody {
    fn from(value: Withdrawal) -> Self {
        TransactionBody::Withdrawal(value)
    }
}

#[derive(AsnType, Encode, Decode, Debug)]
#[non_exhaustive]
pub struct Deposit {
    pub amount: Amount,
}

impl Deposit {
    pub fn new(amount: impl Into<Amount>) -> Self {
        Self {
            amount: amount.into(),
        }
    }
}

#[derive(AsnType, Encode, Decode, Debug)]
#[non_exhaustive]
pub struct Transfer {
    pub recipient: PublicKey,
    pub amount: Amount,
}

impl Transfer {
    pub fn new(recipient: impl Into<PublicKey>, amount: impl Into<Amount>) -> Self {
        Self {
            recipient: recipient.into(),
            amount: amount.into(),
        }
    }
}

#[derive(AsnType, Encode, Decode, Debug)]
#[non_exhaustive]
pub struct Withdrawal {
    pub amount: Amount,
}

impl Withdrawal {
    pub fn new(amount: impl Into<Amount>) -> Self {
        Self {
            amount: amount.into(),
        }
    }
}

impl TryFrom<&[u8]> for SigningPayload {
    type Error = TxError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        SigningPayload::der_decode(value)
    }
}

impl TryFrom<SigningPayload> for Vec<u8> {
    type Error = TxError;

    fn try_from(value: SigningPayload) -> Result<Self, Self::Error> {
        value.der_encode()
    }
}

#[cfg(test)]
mod tests {
    use crate::asn::{Amount, SigningPayload};

    #[test]
    fn test_encode_deposit() {
        const AMOUNT: u64 = 1000;
        let encoded = SigningPayload::new_deposit(AMOUNT).der_encode().unwrap();

        assert_eq!(
            encoded,
            vec![
                0b00110000, // T: 0b00 <- universal, 0b1 <- constructed, 0b10000 (16) <- SEQUENCE tag
                0b00001001, // L: 0b0 <- short form, 0b0001100 (9) <- length
                0b00000010, // T: 0b00 <- universal, 0b0 <- primitive, 0b00010 (2) <- INTEGER tag
                0b00000001, // L: 0b0 <- short form, 0b0000001 (1) <- length
                0b00000000, // V: 0b00000000 (0) <- value
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
        let encoded = SigningPayload::new_transfer(NONCE, RECIPIENT, AMOUNT)
            .der_encode()
            .unwrap();

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
        let encoded = SigningPayload::new_withdrawal(NONCE, AMOUNT)
            .der_encode()
            .unwrap();

        assert_eq!(encoded, vec![48, 9, 2, 1, 1, 162, 4, 2, 2, 3, 232]);
    }

    #[test]
    fn test_hex_encode_nixos_end_to_end_payloads() {
        fn hex_encode(payload: SigningPayload) -> String {
            hex::encode(payload.der_encode().unwrap())
        }

        let deposit_payload = hex_encode(SigningPayload::new_deposit(1000));
        assert_eq!(deposit_payload.as_str(), "3009020100a004020203e8");

        let transfer_payload = hex_encode(SigningPayload::new_transfer(
            0,
            "bob_public_key".as_bytes(),
            Amount::from(1000),
        ));
        assert_eq!(
            transfer_payload.as_str(),
            "3019020100a114040e626f625f7075626c69635f6b6579020203e8"
        );

        let withdrawal_payload = hex_encode(SigningPayload::new_withdrawal(1, 1000));
        assert_eq!(withdrawal_payload.as_str(), "3009020101a204020203e8");
    }
}
