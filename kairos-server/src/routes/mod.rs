pub mod deposit;
pub mod transfer;
pub mod withdraw;

pub use deposit::deposit_handler;
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
