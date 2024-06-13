pub mod submit_batch;
pub mod transactions;
mod trie;

use std::collections::HashSet;
use std::{sync::Arc, thread};

use tokio::{
    sync::{mpsc, RwLock},
    task,
};

use casper_client::types::DeployHash;

pub use self::trie::TrieStateThreadMsg;
use crate::{config::ServerConfig, state::submit_batch::submit_proof_to_contract};
use kairos_circuit_logic::transactions::KairosTransaction;
use kairos_trie::{stored::memory_db::MemoryDb, NodeHash, TrieRoot};

use kairos_data::Pool;

pub type ServerState = Arc<ServerStateInner>;

pub struct ServerStateInner {
    pub batch_state_manager: BatchStateManager,
    pub server_config: ServerConfig,
    pub known_deposit_deploys: RwLock<HashSet<DeployHash>>,
    pub pool: Pool,
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
    pub fn new(config: &ServerConfig, db: trie::Database, batch_root: TrieRoot<NodeHash>) -> Self {
        let batch_config = config.batch_config.clone();
        let casper_rpc = config.casper_rpc.clone();
        let contract_hash = config.kairos_demo_contract_hash;

        let secret_key = config
            .secret_key_file
            .as_ref()
            // We already checked that we can read the secret key in at startup.
            // SecretKey does not implement Clone, so we need to clone the path and read it again.
            .map(|f| casper_client_types::SecretKey::from_file(f).expect("Invalid secret key"));

        let (queued_transactions, txn_receiver) = mpsc::channel(1000);
        // This queue provides back pressure to the trie thread.
        let (batch_sender, mut batch_rec) = mpsc::channel(10);
        let trie_thread = trie::spawn_state_thread(
            config.batch_config.clone(),
            txn_receiver,
            batch_sender,
            db,
            batch_root,
        );

        let batch_output_handler = tokio::spawn(async move {
            while let Some(batch_output) = batch_rec.recv().await {
                tracing::info!(
                    "Sending batch output to proving server: {:?}",
                    batch_output.proof_inputs.transactions
                );

                let prove_url = batch_config
                    .proving_server
                    .join("/api/v1/prove/batch")
                    .expect("Invalid URL");

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
                    tracing::info!("Proving server returned success");
                    let proof_serialized = res.bytes().await.unwrap_or_else(|e| {
                        tracing::error!("Could not read response from proving server: {}", e);
                        panic!("Could not read response from proving server: {}", e);
                    });

                    if let Some(secret_key) = secret_key.as_ref() {
                        submit_proof_to_contract(
                            secret_key,
                            contract_hash,
                            casper_rpc.clone(),
                            proof_serialized.to_vec(),
                        )
                        .await
                    } else {
                        tracing::warn!("No secret key provided. Not submitting proof to contract.");
                    }
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
    pub fn new_empty(config: &ServerConfig) -> Self {
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
