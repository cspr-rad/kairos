#![no_std]
use kairos_trie::{
    stored::{memory_db::MemoryDb, merkle::{SnapshotBuilder, Snapshot},},
    KeyHash, Transaction, TrieRoot, NodeHash
};

extern crate alloc;
use alloc::vec::Vec;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemoCircuitInput{
    pub batch: Vec<Operation>,
    pub snapshot: Snapshot<[u8; 8]>,
    pub new_root_hash: TrieRoot<NodeHash>,
    pub old_root_hash: TrieRoot<NodeHash>,
}

pub type Value = [u8; 8];

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Operation {
    Get(KeyHash),
    Insert(KeyHash, Value),
    EntryGet(KeyHash),
    EntryInsert(KeyHash, Value),
    EntryAndModifyOrInsert(KeyHash, Value),
    EntryOrInsert(KeyHash, Value),
}