use std::{
    mem,
    thread::{self, JoinHandle},
};

use sha2::Sha256;
use tokio::sync::{mpsc, oneshot};

use super::transactions::{
    batch_state::{Account, BatchState, BatchedTxns},
    Signed, Transaction,
};
use crate::AppErr;
use kairos_trie::{
    stored::{
        memory_db::MemoryDb,
        merkle::{Snapshot, SnapshotBuilder},
    },
    DigestHasher, NodeHash, TrieRoot,
};

pub type Database = MemoryDb<Account>;

pub enum TrieStateThreadMsg {
    Transaction(Signed<Transaction>),
    Commit(oneshot::Sender<Result<BatchOutput, AppErr>>),
}

pub fn spawn_state_thread(
    mut queue: mpsc::Receiver<TrieStateThreadMsg>,
    db: Database,
    batch_epoch: u64,
    batch_root: TrieRoot<NodeHash>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut state = TrieState::new(db, batch_epoch, batch_root);

        while let Some(msg) = queue.blocking_recv() {
            match msg {
                TrieStateThreadMsg::Transaction(txn) => {
                    if let Err(err) = state.batched_state.transaction(txn) {
                        // Transactions should be validated before being added to queue
                        tracing::error!("THIS IS A BUG transaction failed: {:?}", err);
                    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchOutput {
    pub batch_epoch: u64,
    pub new_root: TrieRoot<NodeHash>,
    pub old_root: TrieRoot<NodeHash>,
    pub snapshot: Snapshot<Account>,
    pub batched_txns: BatchedTxns,
}

pub struct TrieState {
    db: Database,
    batch_root: TrieRoot<NodeHash>,
    batched_state:
        BatchState<kairos_trie::Transaction<SnapshotBuilder<Database, Account>, Account>>,
}

impl TrieState {
    pub fn new(db: Database, batch_epoch: u64, batch_root: TrieRoot<NodeHash>) -> Self {
        let trie_txn = kairos_trie::Transaction::from_snapshot_builder(
            SnapshotBuilder::<_, Account>::empty(db.clone()).with_trie_root_hash(batch_root),
        );

        Self {
            db,
            batch_root,
            batched_state: BatchState::new(batch_epoch, trie_txn),
        }
    }

    pub fn commit_and_start_new_txn(&mut self) -> Result<BatchOutput, AppErr> {
        let old_epoch = self.batched_state.batch_epoch;
        let old_trie_txn = &self.batched_state.kv_db;
        let old_root = self.batch_root;
        let new_root = old_trie_txn.commit(&mut DigestHasher::<Sha256>::default())?;

        let snapshot = old_trie_txn.build_initial_snapshot();
        let new_trie_txn = kairos_trie::Transaction::from_snapshot_builder(
            SnapshotBuilder::<_, Account>::empty(self.db.clone()).with_trie_root_hash(new_root),
        );

        let old_batch_state = mem::replace(
            &mut self.batched_state,
            BatchState::new(old_epoch + 1, new_trie_txn),
        );
        self.batch_root = new_root;

        Ok(
            BatchOutput {
                batch_epoch: old_epoch,
                new_root,
                old_root,
                snapshot,
                batched_txns: old_batch_state.batched_txns,
            },
            // New TrieState at epoch + 1
        )
    }
}
