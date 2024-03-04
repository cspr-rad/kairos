use crate::asn;
use crate::error::TxError;

use num_traits::cast::ToPrimitive;

// Converts an ASN.1 decoded public key into raw byte representation.
impl From<asn::PublicKey> for Vec<u8> {
    fn from(value: asn::PublicKey) -> Self {
        value.0.into()
    }
}

// Attempts to convert an ASN.1 decoded amount (which is represented as a big integer)
// into a `u64`. This conversion can fail if the decoded value is outside the `u64` range,
// thereby enforcing the specified ASN.1 constraints on the value: `INTEGER
// (0..18446744073709551615)`.
impl TryFrom<asn::Amount> for u64 {
    type Error = TxError;

    fn try_from(value: asn::Amount) -> Result<Self, Self::Error> {
        value
            .0
            .to_u64()
            .ok_or(TxError::ConstraintViolation { field: "amount" })
    }
}

// Similar to `asn::Amount`, attempts to convert an ASN.1 decoded nonce into a `u64`.
// This is crucial for validating that the nonce adheres to expected constraint:
// `INTEGER (0..18446744073709551615)`.
impl TryFrom<asn::Nonce> for u64 {
    type Error = TxError;

    fn try_from(value: asn::Nonce) -> Result<Self, Self::Error> {
        value
            .0
            .to_u64()
            .ok_or(TxError::ConstraintViolation { field: "nonce" })
    }
}
