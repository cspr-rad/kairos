/*
        The deposit contract requires a Casper-typed Deposit struct.
        We can either keep this contract-types crate or move the deposit struct elsewhere.
*/

use casper_types::{Key, U512};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Deposit {
    pub account: Key,
    pub amount: U512,
    pub timestamp: Option<String>,
    pub processed: bool,
}
