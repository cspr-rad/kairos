pub mod transactions;
mod trie;

use std::{sync::Arc, thread};
use tokio::{sync::mpsc, task};

pub use self::trie::TrieStateThreadMsg;
use crate::config::{BatchConfig, ServerConfig};
use crate::deposit_manager::DepositManager;
use kairos_circuit_logic::transactions::KairosTransaction;
use kairos_trie::{stored::memory_db::MemoryDb, NodeHash, TrieRoot};

pub type ServerState = Arc<ServerStateInner>;

pub struct ServerStateInner {
    pub batch_state_manager: BatchStateManager,
    pub server_config: ServerConfig,
    pub deposit_manager: Option<DepositManager>,
}

/// The `BatchStateManager` is a piece of Axum state.
/// It is the entry point for interacting with the trie.
///
/// Messages are sent to the trie thread via the `queued_transactions` channel.
/// The trie thread processes these messages and sends responses back to the caller
/// via a oneshot channel in each `TrieStateThreadMsg`.
#[derive(Debug)]
pub struct BatchStateManager {
    pub trie_thread: thread::JoinHandle<()>,
    pub batch_output_handler: task::JoinHandle<()>,
    pub queued_transactions: mpsc::Sender<TrieStateThreadMsg>,
}

impl BatchStateManager {
    /// Create a new `BatchStateManager` with the given `db` and `batch_root`.
    /// `batch_root` and it's descendants must be in the `db`.
    /// This method spawns the trie state thread, it should be called only once.
    pub fn new(config: BatchConfig, db: trie::Database, batch_root: TrieRoot<NodeHash>) -> Self {
        let (queued_transactions, txn_receiver) = mpsc::channel(1000);
        // This queue provides back pressure to the trie thread.
        let (batch_sender, mut batch_rec) = mpsc::channel(10);
        let trie_thread =
            trie::spawn_state_thread(config.clone(), txn_receiver, batch_sender, db, batch_root);

        let batch_output_handler = tokio::spawn(async move {
            while let Some(batch_output) = batch_rec.recv().await {
                let prove_url = config.proving_server.join("prove").expect("Invalid URL");

                let res = reqwest::Client::new()
                    .post(prove_url)
                    .json(&batch_output.proof_inputs)
                    .send()
                    .await
                    .unwrap_or_else(|e| {
                        tracing::error!("Could not send batch output to proving server: {}", e);
                        panic!("Could not send batch output to proving server: {}", e);
                    });

                if res.status().is_success() {
                    // TODO send the proof to layer 1
                } else {
                    tracing::error!("Proving server returned an error: {:?}", res);
                    panic!("Proving server returned an error: {:?}", res);
                }
            }
        });

        Self {
            trie_thread,
            batch_output_handler,
            queued_transactions,
        }
    }

    /// Create a new `BatchStateManager` with an empty `MemoryDb` and an empty `TrieRoot`.
    /// This is useful for testing.
    pub fn new_empty(config: BatchConfig) -> Self {
        Self::new(config, MemoryDb::empty(), TrieRoot::default())
    }

    pub async fn enqueue_transaction(&self, txn: KairosTransaction) -> Result<(), crate::AppErr> {
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
