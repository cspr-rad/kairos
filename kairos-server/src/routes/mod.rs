pub mod deposit;
#[cfg(feature = "deposit-mock")]
pub mod deposit_mock;
pub mod get_nonce;
pub mod transfer;
pub mod withdraw;

pub use deposit::deposit_handler;
#[cfg(feature = "deposit-mock")]
pub use deposit_mock::deposit_mock_handler;
pub use get_nonce::get_nonce_handler;
pub use transfer::transfer_handler;
pub use withdraw::withdraw_handler;

use crate::utils::{hex_to_vec, vec_to_hex};
use crate::{PublicKey, Signature};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PayloadBody {
    #[serde(deserialize_with = "hex_to_vec", serialize_with = "vec_to_hex")]
    pub public_key: PublicKey,
    #[serde(deserialize_with = "hex_to_vec", serialize_with = "vec_to_hex")]
    pub payload: Vec<u8>,
    #[serde(deserialize_with = "hex_to_vec", serialize_with = "vec_to_hex")]
    pub signature: Signature,
}
