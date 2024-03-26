use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use tokio::sync::RwLock;

use kairos_tx::asn::{Deposit, Transfer, Withdrawal};

use crate::PublicKey;

pub type LockedBatchState = Arc<RwLock<BatchState>>;

#[derive(Debug)]
pub struct BatchState {
    pub balances: HashMap<PublicKey, u64>,
    pub batch_epoch: u64,
    /// The set of transfers that will be batched in the next epoch.
    pub batched_transfers: HashSet<Transfer>,
    pub batched_deposits: Vec<Deposit>,
    pub batched_withdrawals: Vec<Withdrawal>,
}
impl BatchState {
    pub fn new() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            balances: HashMap::new(),
            batch_epoch: 0,
            batched_transfers: HashSet::new(),
            batched_deposits: Vec::new(),
            batched_withdrawals: Vec::new(),
        }))
    }

    fn transfer(&mut self, from: &PublicKey, transfer: Signed<Transfer>) -> Result<(), AppErr> {
        if transfer.transaction.amount == 0 {
            return Err(AppErr::set_status(
                anyhow!("transfer amount must be greater than 0"),
                StatusCode::BAD_REQUEST,
            ));
        }

        tracing::info!("TODO: verifying transfer signature");

        // We pre-check this read-only to error early without acquiring the write lock.
        // This prevents a DoS attack exploiting the write lock.
        tracing::info!("verifying transfer sender has sufficient funds");
        let from_balance = self.balances.get(from).ok_or_else(|| {
            AppErr::set_status(
                anyhow!("Sender does not have an account"),
                StatusCode::BAD_REQUEST,
            )
        })?;

        from_balance
            .checked_sub(transfer.transaction.amount)
            .ok_or_else(|| {
                AppErr::set_status(
                    anyhow!(
                        "Sender does not have sufficient funds, balance={}, transfer_amount={}",
                        from_balance,
                        transfer.transaction.amount
                    ),
                    StatusCode::FORBIDDEN,
                )
            })?;

        let to_balance = self
            .balances
            .get(&transfer.transaction.recipient)
            .unwrap_or(&0);
        if to_balance
            .checked_add(transfer.transaction.amount)
            .is_none()
        {
            return Err(AppErr::set_status(
                anyhow!("Receiver balance overflow"),
                StatusCode::CONFLICT,
            ));
        }

        let from_balance = self.balances.get_mut(from).ok_or_else(|| {
            AppErr::set_status(
                anyhow!(
                    "Sender no longer has an account.
                The sender just removed all their funds."
                ),
                StatusCode::CONFLICT,
            )
        })?;

        *from_balance = from_balance
            .checked_sub(transfer.transaction.amount)
            .ok_or_else(|| {
                AppErr::set_status(
                    anyhow!(
                        "Sender no longer has sufficient funds, balance={}, transfer_amount={}.
                The sender just moved their funds in a concurrent request",
                        from_balance,
                        transfer.transaction.amount
                    ),
                    StatusCode::CONFLICT,
                )
            })?;

        let to_balance = self
            .balances
            .entry(transfer.transaction.recipient.clone())
            .or_insert_with(|| {
                tracing::info!("creating new account for receiver");
                0
            });

        *to_balance = to_balance
            .checked_add(transfer.transaction.amount)
            .ok_or_else(|| {
                AppErr::set_status(anyhow!("Receiver balance overflow"), StatusCode::CONFLICT)
            })?;

        Ok(())
    }
}

// We could use a self-referential struct here to avoid Arc, but it's not worth the complexity
#[derive(Debug, Default)]
pub struct BatchedTxns {
    set: HashSet<Arc<Transaction>>,
    ord: Vec<Arc<Transaction>>,
}

impl BatchedTxns {
    pub fn new() -> Self {
        BatchedTxns::default()
    }

    fn insert(&mut self, txn: Transaction) -> bool {
        let txn = Arc::new(txn);
        let new = self.set.insert(txn.clone());
        if new {
            self.ord.push(txn);
        };

        new
    }

    fn get(&self, op_idx: usize) -> Option<&Arc<Transaction>> {
        self.ord.get(op_idx)
    }
}
