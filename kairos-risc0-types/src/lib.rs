use serde::{Serialize, Deserialize};
//use risc0_zkvm::Receipt;

#[cfg(feature = "tornado-tree")]
pub use tornado_tree_rs::{TornadoTree, crypto::hash_bytes};

pub use casper_types::{bytesrepr::ToBytes, Key, U512};
use serde_json;
use std::collections::HashMap;
mod tests;
pub mod constants;
// The decision to use "Key" over  more deterministic types is based on the variable design
// with respect to target node architecture. Everything will be handled in Bytes and Keys.

pub trait HashableStruct{
    fn hash(&self) -> Vec<u8>;
}

/*#[derive(Serialize, Deserialize)]
pub struct RiscZeroProof{
    pub receipt: Receipt,
    pub program_id: Vec<u32>
}*/

#[cfg(feature = "tornado-tree")]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TransactionBatch{
    // for first iteration of tests: String is just index
    // Will be tested with 1 Deposit, where index is "0"
    // and one transfer where index is "1"
    pub deposits: Vec<Deposit>,
    pub transfers : Vec<Transfer>,
    pub withdrawals: Vec<Withdrawal>
}

// This will likely not be used, since hashing the Balance state will be sufficient
// New leaf in the tree <- New Balance state
#[cfg(feature = "tornado-tree")]
impl HashableStruct for TransactionBatch{
    fn hash(&self) -> Vec<u8>{
        hash_bytes(serde_json::to_vec(self).unwrap())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Deposit {
        pub account: Key,
        pub amount: U512,
        pub timestamp: Option<u32>,
        pub processed: bool,
}
#[cfg(feature = "tornado-tree")]
impl HashableStruct for Deposit{
    fn hash(&self) -> Vec<u8>{
        hash_bytes(serde_json::to_vec(self).unwrap())
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Withdrawal {
        pub account: Key,
        pub amount: U512,
        pub timestamp: u32,
       pub  processed: bool,
}
#[cfg(feature = "tornado-tree")]
impl HashableStruct for Withdrawal{
    fn hash(&self) -> Vec<u8>{
        hash_bytes(serde_json::to_vec(self).unwrap())
    }
}

#[cfg(feature = "tornado-tree")]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transfer {
        sender: Key,
        recipient: Key,
        amount: U512,
        timestamp: u32,
        signature: Vec<u8>,
        processed: bool,
        nonce: u64
}
#[cfg(feature = "tornado-tree")]
impl HashableStruct for Transfer{
    fn hash(&self) -> Vec<u8>{
        hash_bytes(serde_json::to_vec(self).unwrap())
    }
}
#[cfg(feature = "tornado-tree")]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CircuitArgs{
    pub tornado: TornadoTree,
    pub batch: TransactionBatch
}

#[cfg(feature = "tornado-tree")]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CircuitJournal{
    pub input: CircuitArgs,
    pub output: Option<TornadoTree>
}