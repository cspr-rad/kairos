#![no_std]
pub mod verification_logic;

use kairos_trie::{
    stored::merkle::Snapshot,
    TrieRoot, NodeHash
};

extern crate alloc;
use alloc::vec::Vec;

use serde::{Serialize, Deserialize};

pub use kairos_common_types::transactions;
use verification_logic::Account;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemoCircuitInput{
    pub batch: Vec<transactions::Signed<transactions::Transaction>>,
    pub snapshot: Snapshot<Account>,
    pub new_root_hash: TrieRoot<NodeHash>,
    pub old_root_hash: TrieRoot<NodeHash>,
}
