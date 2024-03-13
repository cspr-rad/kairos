use serde::{Serialize, Deserialize};
use casper_types::{Key, U512, URef};
use std::collections::HashMap;
mod tests;
pub mod constants;

#[cfg(feature = "kairos-delta-tree")]
pub use kairos_delta_tree::{KairosDeltaTree, crypto::hash_bytes};
// The decision to use "Key" over  more deterministic types is based on the variable design
// with respect to target node architecture. Everything will be handled in Bytes and Keys.

pub trait HashableStruct{
    fn hash(&self) -> Vec<u8>;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RiscZeroProof{
    pub receipt_serialized: Vec<u8>,
    pub program_id: Vec<u32>
}

#[cfg(feature = "kairos-delta-tree")]
#[derive(Serialize,Deserialize, Debug, Clone, PartialEq)]
pub struct TransactionBatch{
    pub deposits: Vec<Deposit>,
    pub transfers : Vec<Transfer>,
    pub withdrawals: Vec<Withdrawal>
}
// Hash the Batch as a struct
#[cfg(feature = "kairos-delta-tree")]
impl HashableStruct for TransactionBatch{
    fn hash(&self) -> Vec<u8>{
        hash_bytes(serde_json_wasm::to_vec(self).unwrap())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Deposit {
        pub account: Key,
        pub amount: U512,
        pub timestamp: Option<String>,
        pub processed: bool,
}
#[cfg(feature = "kairos-delta-tree")]
impl HashableStruct for Deposit{
    fn hash(&self) -> Vec<u8>{
        hash_bytes(serde_json_wasm::to_vec(self).unwrap())
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Withdrawal {
        pub account: Key,
        pub amount: U512,
        pub timestamp: String,
       pub  processed: bool,
}
#[cfg(feature = "kairos-delta-tree")]
impl HashableStruct for Withdrawal{
    fn hash(&self) -> Vec<u8>{
        hash_bytes(serde_json_wasm::to_vec(self).unwrap())
    }
}

#[cfg(feature = "kairos-delta-tree")]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transfer {
        pub sender: Key,
        pub recipient: Key,
        pub amount: U512,
        pub timestamp: String,
        pub signature: Vec<u8>,
        pub processed: bool,
        pub nonce: u64
}
#[cfg(feature = "kairos-delta-tree")]
impl HashableStruct for Transfer{
    fn hash(&self) -> Vec<u8>{
        hash_bytes(serde_json_wasm::to_vec(self).unwrap())
    }
}
#[cfg(feature = "kairos-delta-tree")]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CircuitArgs{
    pub tree: KairosDeltaTree,
    pub batch: TransactionBatch
}

#[cfg(feature = "kairos-delta-tree")]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CircuitJournal{
    pub input: KairosDeltaTree,
    pub output: KairosDeltaTree
}