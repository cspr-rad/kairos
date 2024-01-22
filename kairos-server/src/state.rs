use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use tokio::sync::RwLock;

use crate::{
    routes::{deposit::Deposit, transfer::Transfer, withdraw::Withdrawal},
    PublicKey,
};

pub type LockedBatchState = Arc<RwLock<BatchState>>;

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
}
