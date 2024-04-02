use std::{
    collections::HashSet,
    sync::Arc,
    thread::{self, JoinHandle},
};

use anyhow::anyhow;
use sha2::{digest::FixedOutputReset, Digest, Sha256};
use tokio::sync::{mpsc, oneshot};

use kairos_trie::{
    stored::{
        memory_db::MemoryDb,
        merkle::{Snapshot, SnapshotBuilder},
    },
    DigestHasher, KeyHash, NodeHash, PortableHash, PortableUpdate, TrieRoot,
};

use crate::{AppErr, PublicKey};

use super::transactions::{Deposit, Signed, Transaction, Transfer, Withdraw};

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
                    if let Err(err) = state.transaction(txn) {
                        // Transactions should be validated before being added to queue
                        tracing::error!("THIS IS A BUG transaction failed: {:?}", err);
                    }
                }
                TrieStateThreadMsg::Commit(sender) => {
                    let result = state.commit_and_start_new_txn();
                    if let Err(err) = sender.send(result) {
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
    batch_epoch: u64,
    batch_root: TrieRoot<NodeHash>,
    batched_txns: BatchedTxns,

    trie_txn: kairos_trie::Transaction<SnapshotBuilder<Database, Account>, Account>,
}

impl TrieState {
    pub fn new(db: Database, batch_epoch: u64, batch_root: TrieRoot<NodeHash>) -> Self {
        let trie_txn = kairos_trie::Transaction::from_snapshot_builder(
            SnapshotBuilder::<_, Account>::empty(db.clone()).with_trie_root_hash(batch_root),
        );

        Self {
            db,
            batch_epoch,
            batch_root,
            batched_txns: BatchedTxns::new(),
            trie_txn,
        }
    }

    pub fn commit_and_start_new_txn(&mut self) -> Result<BatchOutput, AppErr> {
        let new_root = self
            .trie_txn
            .commit(&mut DigestHasher::<Sha256>::default())?;

        let snapshot = self.trie_txn.build_initial_snapshot();

        self.trie_txn = kairos_trie::Transaction::from_snapshot_builder(
            SnapshotBuilder::<_, Account>::empty(self.db.clone()).with_trie_root_hash(new_root),
        );

        Ok(BatchOutput {
            batch_epoch: self.batch_epoch,
            new_root,
            old_root: self.batch_root,
            snapshot,
            batched_txns: std::mem::take(&mut self.batched_txns),
        })
    }

    fn transaction(&mut self, txn: Signed<Transaction>) -> Result<(), AppErr> {
        let Signed {
            public_key,
            epoch,
            nonce: _,
            transaction,
        } = txn;

        if epoch != self.batch_epoch {
            return Err(anyhow!("transaction epoch does not match batch epoch").into());
        }

        if !self.batched_txns.contains(&transaction) {
            return Err(anyhow!("transaction already in batch").into());
        }

        let amount = match &transaction {
            Transaction::Transfer(transfer) => transfer.amount,
            Transaction::Deposit(deposit) => deposit.amount,
            Transaction::Withdraw(withdraw) => withdraw.amount,
        };

        if amount == 0 {
            return Err(anyhow!("transaction amount must be greater than 0").into());
        }

        match transaction {
            Transaction::Transfer(ref transfer) => self.transfer(&public_key, transfer),
            Transaction::Deposit(ref deposit) => self.deposit(&public_key, deposit),
            Transaction::Withdraw(ref withdraw) => self.withdraw(&public_key, withdraw),
        }?;

        self.batched_txns.insert(transaction);

        Ok(())
    }

    /// Avoid calling this method with a transfer that will fail.
    /// This method's prechecks may cause the Snapshot to bloat with unnecessary data if the transfer fails.
    fn transfer(&mut self, sender: &PublicKey, transfer: &Transfer) -> Result<(), AppErr> {
        let [sender_hash, recipient_hash] =
            hash_buffers([sender.as_slice(), transfer.recipient.as_slice()]);

        let sender_account = self
            .trie_txn
            .get(&sender_hash)?
            .ok_or_else(|| anyhow!("Sender does not have an account"))?;

        if sender_account.pubkey != *sender {
            return Err(anyhow!("hash collision detected on sender account").into());
        }

        if sender_account.balance < transfer.amount {
            return Err(anyhow!("sender has insufficient funds").into());
        }

        if let Some(recipient_account) = self.trie_txn.get(&recipient_hash)? {
            if recipient_account.pubkey != transfer.recipient {
                return Err(anyhow!("hash collision detected on recipient account").into());
            }

            if recipient_account
                .balance
                .checked_add(transfer.amount)
                .is_none()
            {
                return Err(anyhow!("recipient balance overflow").into());
            }
        }

        tracing::info!("applying transfer");
        let sender_account = self.trie_txn.entry(&sender_hash)?.or_insert_with(|| {
            unreachable!("sender account should exist");
        });

        // Remove the amount from the sender's account
        sender_account.balance = sender_account
            .balance
            .checked_sub(transfer.amount)
            .expect("Sender balance underflow");

        let recipient_account = self.trie_txn.entry(&recipient_hash)?.or_insert_with(|| {
            tracing::info!("creating new account for recipient");
            Account::new(transfer.recipient.clone(), 0)
        });

        // Add the amount to the recipient's account
        recipient_account.balance = recipient_account
            .balance
            .checked_add(transfer.amount)
            .expect("recipient balance overflow");

        Ok(())
    }

    fn deposit(&mut self, recipient: &PublicKey, deposit: &Deposit) -> Result<(), AppErr> {
        tracing::info!("verifying the deposit can be applied");

        let [recipient_hash] = hash_buffers([recipient.as_slice()]);

        let recipient_account = self.trie_txn.entry(&recipient_hash)?.or_insert_with(|| {
            tracing::info!("creating new account for recipient");
            Account::new(recipient.clone(), 0)
        });

        if recipient_account.pubkey != *recipient {
            return Err(anyhow!("hash collision detected on recipient account").into());
        }

        recipient_account.balance = recipient_account
            .balance
            .checked_add(deposit.amount)
            .ok_or_else(|| anyhow!("recipient balance overflow"))?;

        Ok(())
    }

    fn withdraw(&mut self, withdrawer: &PublicKey, withdraw: &Withdraw) -> Result<(), AppErr> {
        tracing::info!("verifying the withdrawal can be applied");

        let [sender_hash] = hash_buffers([withdrawer.as_slice()]);

        let sender_account = self
            .trie_txn
            .get(&sender_hash)?
            .ok_or_else(|| anyhow!("Sender does not have an account"))?;

        if sender_account.pubkey != *withdrawer {
            return Err(anyhow!("hash collision detected on sender account").into());
        }

        if sender_account.balance < withdraw.amount {
            return Err(anyhow!("sender has insufficient funds").into());
        }

        let sender_account = self.trie_txn.entry(&sender_hash)?.or_insert_with(|| {
            unreachable!("sender account should exist");
        });
        sender_account.balance = sender_account
            .balance
            .checked_sub(withdraw.amount)
            .expect("sender balance underflow");

        Ok(())
    }
}

// We could use a self-referential struct here to avoid Arc, but it's not worth the complexity
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct BatchedTxns {
    set: HashSet<Arc<Transaction>>,
    ord: Vec<Arc<Transaction>>,
}

impl BatchedTxns {
    pub fn new() -> Self {
        BatchedTxns::default()
    }

    /// Insert a transaction into the batch.
    /// Returns true if the transaction was not already in the batch.
    fn insert(&mut self, txn: Transaction) -> bool {
        let txn = Arc::new(txn);
        let new = self.set.insert(txn.clone());
        if new {
            self.ord.push(txn);
        };

        new
    }

    /// Check if a transaction is in the batch.
    fn contains(&self, txn: &Transaction) -> bool {
        self.set.contains(txn)
    }

    #[allow(dead_code)]
    fn get(&self, op_idx: usize) -> Option<&Arc<Transaction>> {
        self.ord.get(op_idx)
    }
}

/// A utility function to hash multiple buffers reusing the same hasher.
fn hash_buffers<const N: usize, T: AsRef<[u8]>>(items: [T; N]) -> [KeyHash; N] {
    let mut out = [KeyHash([0; 8]); N];
    let mut hasher = Sha256::new();

    for (i, item) in items.iter().enumerate() {
        hasher.update(item.as_ref());
        out[i] = KeyHash::from_bytes(&hasher.finalize_fixed_reset().into());
    }

    out
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Account {
    pub pubkey: PublicKey,
    pub balance: u64,
}

impl Account {
    pub fn new(pubkey: PublicKey, balance: u64) -> Self {
        Self { pubkey, balance }
    }
}

impl PortableHash for Account {
    fn portable_hash<H: PortableUpdate>(&self, hasher: &mut H) {
        self.pubkey.portable_hash(hasher);
        self.balance.portable_hash(hasher);
    }
}
