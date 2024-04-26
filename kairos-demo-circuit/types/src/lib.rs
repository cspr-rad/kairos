#![no_std]
use kairos_trie::{
    stored::{memory_db::MemoryDb, merkle::{SnapshotBuilder, Snapshot},},
    KeyHash, TrieRoot, NodeHash
};

extern crate alloc;
use alloc::vec::Vec;

use serde::{Serialize, Deserialize};

pub use kairos_common_types::transactions::Transaction;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemoCircuitInput{
    pub batch: Vec<Transaction>,
    pub snapshot: Snapshot<[u8; 8]>,
    pub new_root_hash: TrieRoot<NodeHash>,
    pub old_root_hash: TrieRoot<NodeHash>,
}
