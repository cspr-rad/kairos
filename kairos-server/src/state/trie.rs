use std::{
    mem,
    thread::{self, JoinHandle},
};

use sha2::Sha256;
use tokio::sync::{mpsc, oneshot};

use super::transactions::batch_state::{Account, BatchState};
use crate::AppErr;
use kairos_circuit_logic::transactions::{Signed, Transaction};
use kairos_trie::{
    stored::{
        memory_db::MemoryDb,
        merkle::{Snapshot, SnapshotBuilder},
    },
    DigestHasher, NodeHash, TrieRoot,
};

pub type Database = MemoryDb<Account>;

pub enum TrieStateThreadMsg {
    Transaction(Signed<Transaction>, oneshot::Sender<Result<(), AppErr>>),
    Commit(oneshot::Sender<Result<BatchOutput, AppErr>>),
}

impl TrieStateThreadMsg {
    pub fn transaction(txn: Signed<Transaction>) -> (Self, oneshot::Receiver<Result<(), AppErr>>) {
        let (sender, receiver) = oneshot::channel();
        (Self::Transaction(txn, sender), receiver)
    }

    pub fn commit() -> (Self, oneshot::Receiver<Result<BatchOutput, AppErr>>) {
        let (sender, receiver) = oneshot::channel();
        (Self::Commit(sender), receiver)
    }
}

pub fn spawn_state_thread(
    mut queue: mpsc::Receiver<TrieStateThreadMsg>,
    db: Database,
    batch_root: TrieRoot<NodeHash>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut state = TrieState::new(db, batch_root);

        while let Some(msg) = queue.blocking_recv() {
            match msg {
                TrieStateThreadMsg::Transaction(txn, responder) => {
                    let res = state.batch_state.execute_transaction(txn);

                    responder.send(res).unwrap_or_else(|err| {
                        tracing::warn!(
                            "Transaction submitter hung up before receiving response: {}",
                            err.map(|()| "Success".to_string())
                                .unwrap_or_else(|err| err.to_string())
                        )
                    })
                }
                TrieStateThreadMsg::Commit(sender) => {
                    let res = state.commit_and_start_new_txn();

                    if let Err(err) = sender.send(res) {
                        tracing::error!("failed to send commit result: {:?}", err);
                    }
                }
            }
        }
    })
}

/// Proof input data that is sent to the L1 contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchOutput {
    pub new_root: TrieRoot<NodeHash>,
    pub old_root: TrieRoot<NodeHash>,
    pub snapshot: Snapshot<Account>,
    pub batched_txns: Vec<Signed<Transaction>>,
}

/// A struct for tracking the state of the trie between batches.
///
/// The `TrieStateThread` responds to messages by applying transactions against this struct.
/// When a commit message is received, the trie state is committed and a new trie state is created.
/// Committing the trie state returns a `BatchOutput` which serves as the proof input data for the L1 contract.
pub struct TrieState {
    db: Database,
    /// The root hash of the trie at the start of the current batch.
    batch_root: TrieRoot<NodeHash>,
    batch_state: BatchState<kairos_trie::Transaction<SnapshotBuilder<Database, Account>, Account>>,
}

impl TrieState {
    pub fn new(db: Database, batch_root: TrieRoot<NodeHash>) -> Self {
        let trie_txn = kairos_trie::Transaction::from_snapshot_builder(
            SnapshotBuilder::<_, Account>::empty(db.clone()).with_trie_root_hash(batch_root),
        );

        Self {
            db,
            batch_root,
            batch_state: BatchState::new(trie_txn),
        }
    }

    /// Calculate the new root hash of the trie and sync changes to the database.
    pub fn commit_and_start_new_txn(&mut self) -> Result<BatchOutput, AppErr> {
        let old_trie_txn = &self.batch_state.kv_db;
        let old_root = self.batch_root;
        let new_root = old_trie_txn.commit(&mut DigestHasher::<Sha256>::default())?;

        let snapshot = old_trie_txn.build_initial_snapshot();
        let new_trie_txn = kairos_trie::Transaction::from_snapshot_builder(
            SnapshotBuilder::<_, Account>::empty(self.db.clone()).with_trie_root_hash(new_root),
        );

        let old_batch_state = mem::replace(&mut self.batch_state, BatchState::new(new_trie_txn));
        self.batch_root = new_root;

        Ok(BatchOutput {
            new_root,
            old_root,
            snapshot,
            batched_txns: old_batch_state.batched_txns,
        })
    }
}
