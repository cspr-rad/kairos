use std::rc::Rc;

use anyhow::anyhow;

use crate::{state::trie::Database, AppErr};
use kairos_circuit_logic::{
    account_trie::{Account, AccountTrie},
    transactions::KairosTransaction,
};
use kairos_trie::stored::{merkle::SnapshotBuilder, Store};

/// The state of the batch transaction against the trie.
pub struct BatchState<S: Store<Account>> {
    pub batched_txns: Vec<KairosTransaction>,
    pub account_trie: AccountTrie<S>,
}

impl<S: Store<Account>> BatchState<S> {
    pub fn new(account_trie: AccountTrie<S>) -> Self {
        Self {
            batched_txns: Vec::new(),
            account_trie,
        }
    }
}

impl BatchState<SnapshotBuilder<Rc<Database>, Account>> {
    pub fn execute_transaction(&mut self, txn: KairosTransaction) -> Result<(), AppErr> {
        match txn {
            KairosTransaction::Transfer(ref transfer) => {
                self.account_trie
                    .precheck_transfer(&transfer.public_key, &transfer.transaction, transfer.nonce)
                    .map_err(|err| anyhow!("transfer precheck caught: {err}"))?;

                let _ = self
                    .account_trie
                    .transfer(&transfer.public_key, &transfer.transaction, transfer.nonce)
                    .map_err(|err| panic!("transfer precheck failed to catch: {err}"));
            }
            KairosTransaction::Withdraw(ref withdraw) => {
                self.account_trie
                    .precheck_withdraw(&withdraw.public_key, &withdraw.transaction, withdraw.nonce)
                    .map_err(|err| anyhow!("withdraw precheck caught: {err}"))?;

                let _ = self
                    .account_trie
                    .withdraw(&withdraw.public_key, &withdraw.transaction, withdraw.nonce)
                    .map_err(|err| panic!("withdraw precheck failed to catch: {err}"));
            }

            KairosTransaction::Deposit(ref deposit) => {
                self.account_trie
                    .precheck_deposit(deposit)
                    .map_err(|err| anyhow!("deposit precheck caught: {err}"))?;

                let _ = self
                    .account_trie
                    .deposit(deposit)
                    .map_err(|err| panic!("deposit precheck failed to catch: {err}"));
            }
        }

        self.batched_txns.push(txn);

        Ok(())
    }
}
