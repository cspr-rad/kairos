pub mod transactions;
mod trie;

use std::{sync::Arc, thread::JoinHandle};

use kairos_trie::{stored::memory_db::MemoryDb, NodeHash, TrieRoot};
use tokio::sync::mpsc;

pub use self::trie::TrieStateThreadMsg;

#[derive(Debug)]
pub struct BatchStateManager {
    pub trie_thread: JoinHandle<()>,
    pub queued_transactions: mpsc::Sender<TrieStateThreadMsg>,
}

impl BatchStateManager {
    pub fn new(db: trie::Database, batch_epoch: u64, batch_root: TrieRoot<NodeHash>) -> Arc<Self> {
        let (queued_transactions, receiver) = mpsc::channel(1000);
        let trie_thread = trie::spawn_state_thread(receiver, db, batch_epoch, batch_root);

        Arc::new(Self {
            trie_thread,
            queued_transactions,
        })
    }

    pub fn new_empty() -> Arc<Self> {
        Self::new(MemoryDb::empty(), 0, TrieRoot::default())
    }
}
