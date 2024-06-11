use std::{
    mem,
    rc::Rc,
    thread::{self, JoinHandle},
};

use sha2::Sha256;
use tokio::sync::{mpsc, oneshot};

use super::transactions::batch_state::BatchState;
use crate::AppErr;
use kairos_circuit_logic::{
    account_trie::{Account, AccountTrie},
    transactions::KairosTransaction,
    ProofInputs,
};
use kairos_trie::{
    stored::{memory_db::MemoryDb, merkle::SnapshotBuilder},
    DigestHasher, NodeHash, TrieRoot,
};

pub type Database = MemoryDb<Account>;

#[derive(Debug)]
pub enum TrieStateThreadMsg {
    Transaction(KairosTransaction, oneshot::Sender<Result<(), AppErr>>),
    Commit(oneshot::Sender<Result<BatchOutput, AppErr>>),
}

impl TrieStateThreadMsg {
    pub fn transaction(txn: KairosTransaction) -> (Self, oneshot::Receiver<Result<(), AppErr>>) {
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
            tracing::trace!("Trie State Thread received message: {:?}", msg);
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
    pub proof_inputs: ProofInputs,
}

/// A struct for tracking the state of the trie between batches.
///
/// The `TrieStateThread` responds to messages by applying transactions against this struct.
/// When a commit message is received, the trie state is committed and a new trie state is created.
/// Committing the trie state returns a `BatchOutput` which serves as the proof input data for the L1 contract.
pub struct TrieState {
    db: Rc<Database>,
    /// The root hash of the trie at the start of the current batch.
    batch_root: TrieRoot<NodeHash>,
    batch_state: BatchState<SnapshotBuilder<Rc<Database>, Account>>,
}

impl TrieState {
    pub fn new(db: Database, batch_root: TrieRoot<NodeHash>) -> Self {
        let db = Rc::new(db);

        Self {
            db: db.clone(),
            batch_root,
            batch_state: BatchState::new(AccountTrie::new_try_from_db(db, batch_root)),
        }
    }

    /// Calculate the new root hash of the trie and sync changes to the database.
    pub fn commit_and_start_new_txn(&mut self) -> Result<BatchOutput, AppErr> {
        let old_trie_txn = &self.batch_state.account_trie;
        let old_root = self.batch_root;
        let new_root = old_trie_txn
            .txn
            .commit(&mut DigestHasher::<Sha256>::default())?;

        let snapshot = old_trie_txn.txn.build_initial_snapshot();
        let new_trie_txn = AccountTrie::new_try_from_db(self.db.clone(), new_root);

        let old_batch_state = mem::replace(&mut self.batch_state, BatchState::new(new_trie_txn));
        self.batch_root = new_root;

        Ok(BatchOutput {
            new_root,
            old_root,
            proof_inputs: ProofInputs {
                transactions: old_batch_state.batched_txns.into(),
                trie_snapshot: snapshot,
            },
        })
    }
}
