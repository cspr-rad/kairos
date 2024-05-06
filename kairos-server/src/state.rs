pub mod transactions;
mod trie;

use std::{sync::Arc, thread::JoinHandle};

use tokio::sync::mpsc;

pub use self::trie::TrieStateThreadMsg;
use kairos_trie::{stored::memory_db::MemoryDb, NodeHash, TrieRoot};
use kairos_types::transactions::{Signed, Transaction};

/// The `BatchStateManager` is a piece of Axum state.
/// It is the entry point for interacting with the trie.
///
/// Messages are sent to the trie thread via the `queued_transactions` channel.
/// The trie thread processes these messages and sends responses back to the caller
/// via a oneshot channel in each `TrieStateThreadMsg`.
#[derive(Debug)]
pub struct BatchStateManager {
    pub trie_thread: JoinHandle<()>,
    pub queued_transactions: mpsc::Sender<TrieStateThreadMsg>,
}

impl BatchStateManager {
    /// Create a new `BatchStateManager` with the given `db` and `batch_root`.
    /// `batch_root` and it's descendants must be in the `db`.
    /// This method spawns the trie state thread, it should be called only once.
    pub fn new(db: trie::Database, batch_root: TrieRoot<NodeHash>) -> Arc<Self> {
        let (queued_transactions, receiver) = mpsc::channel(1000);
        let trie_thread = trie::spawn_state_thread(receiver, db, batch_root);

        Arc::new(Self {
            trie_thread,
            queued_transactions,
        })
    }

    /// Create a new `BatchStateManager` with an empty `MemoryDb` and an empty `TrieRoot`.
    /// This is useful for testing.
    pub fn new_empty() -> Arc<Self> {
        Self::new(MemoryDb::empty(), TrieRoot::default())
    }

    pub async fn enqueue_transaction(&self, txn: Signed<Transaction>) -> Result<(), crate::AppErr> {
        let (msg, response) = TrieStateThreadMsg::transaction(txn);

        self.queued_transactions
            .send(msg)
            .await
            .expect("Could not send transaction to trie thread");

        response
            .await
            .expect("Never received response from trie thread")
    }
}
