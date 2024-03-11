use casper_types::{Key, U512};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Deposit {
    pub account: Key,
    pub amount: U512,
}
