mod transactions;
mod trie;

use std::{collections::HashMap, sync::Arc, thread::JoinHandle};

use kairos_trie::{stored::memory_db::MemoryDb, NodeHash, TrieRoot};
use tokio::sync::{mpsc, RwLock};

use crate::PublicKey;

use self::trie::TrieStateThreadMsg;

pub type LockedBatchState = Arc<RwLock<BatchState>>;

#[derive(Debug)]
pub struct BatchState {
    // TODO replace with a database
    pub balances: HashMap<PublicKey, u64>,
    pub batch_epoch: u64,

    pub trie_thread: JoinHandle<()>,
    pub queued_transactions: mpsc::Sender<TrieStateThreadMsg>,
}

impl BatchState {
    pub fn new(
        db: trie::Database,
        balances: HashMap<PublicKey, u64>,
        batch_epoch: u64,
        batch_root: TrieRoot<NodeHash>,
    ) -> Arc<RwLock<Self>> {
        let (queued_transactions, receiver) = mpsc::channel(1000);
        let trie_thread = trie::spawn_state_thread(receiver, db, batch_epoch, batch_root);

        Arc::new(RwLock::new(Self {
            balances,
            batch_epoch,
            trie_thread,
            queued_transactions,
        }))
    }

    pub fn new_empty() -> Arc<RwLock<Self>> {
        Self::new(MemoryDb::empty(), HashMap::new(), 0, TrieRoot::default())
    }
}
