use anyhow::anyhow;

use sha2::{digest::FixedOutputReset, Digest, Sha256};

use crate::{AppErr, PublicKey};
use kairos_circuit_logic::transactions::{Deposit, Signed, Transaction, Transfer, Withdraw};
use kairos_trie::{stored::Store, KeyHash, PortableHash, PortableUpdate};

/// The state of the batch transaction against the trie.
#[derive(Debug)]
pub struct BatchState<DB> {
    pub batched_txns: Vec<Signed<Transaction>>,

    /// A transactional key value store like `kairos_trie::Transaction`.
    pub kv_db: DB,
}

impl<S: Store<Account>> BatchState<kairos_trie::Transaction<S, Account>> {
    pub fn new(kv_db: kairos_trie::Transaction<S, Account>) -> Self {
        Self {
            batched_txns: Vec::new(),
            kv_db,
        }
    }

    pub fn execute_transaction(&mut self, txn: Signed<Transaction>) -> Result<(), AppErr> {
        let amount = match &txn.transaction {
            Transaction::Transfer(transfer) => transfer.amount,
            Transaction::Deposit(deposit) => deposit.amount,
            Transaction::Withdraw(withdraw) => withdraw.amount,
        };

        if amount == 0 {
            return Err(anyhow!("transaction amount must be greater than 0").into());
        }

        match txn.transaction {
            Transaction::Transfer(ref transfer) => {
                self.transfer(&txn.public_key, transfer, txn.nonce)?
            }
            Transaction::Deposit(ref deposit) => self.deposit(&txn.public_key, deposit)?,
            Transaction::Withdraw(ref withdraw) => {
                self.withdraw(&txn.public_key, withdraw, txn.nonce)?
            }
        }

        self.batched_txns.push(txn);

        Ok(())
    }

    /// Avoid calling this method with a transfer that will fail.
    /// This method's prechecks may cause the Snapshot to bloat with unnecessary data if the transfer fails.
    pub fn transfer(
        &mut self,
        sender: &PublicKey,
        transfer: &Transfer,
        nonce: u64,
    ) -> Result<(), AppErr> {
        let [sender_hash, recipient_hash] =
            hash_buffers([sender.as_slice(), transfer.recipient.as_slice()]);

        let sender_account = self
            .kv_db
            .get(&sender_hash)?
            .ok_or_else(|| anyhow!("Sender does not have an account"))?;

        if sender_account.pubkey != *sender {
            return Err(anyhow!("hash collision detected on sender account").into());
        }

        sender_account.check_nonce(nonce)?;

        if sender_account.balance < transfer.amount {
            return Err(anyhow!("sender has insufficient funds").into());
        }

        if let Some(recipient_account) = self.kv_db.get(&recipient_hash)? {
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
        let sender_account = self.kv_db.entry(&sender_hash)?.or_insert_with(|| {
            unreachable!("sender account should exist");
        });

        // Remove the amount from the sender's account
        sender_account.balance = sender_account
            .balance
            .checked_sub(transfer.amount)
            .expect("Sender balance underflow");

        sender_account.increment_nonce();

        let recipient_account = self.kv_db.entry(&recipient_hash)?.or_insert_with(|| {
            tracing::info!("creating new account for recipient");
            Account::new(transfer.recipient.clone(), 0, 0)
        });

        // Add the amount to the recipient's account
        recipient_account.balance = recipient_account
            .balance
            .checked_add(transfer.amount)
            .expect("recipient balance overflow");

        Ok(())
    }

    pub fn deposit(&mut self, recipient: &PublicKey, deposit: &Deposit) -> Result<(), AppErr> {
        tracing::info!("verifying the deposit can be applied");

        let [recipient_hash] = hash_buffers([recipient.as_slice()]);

        let recipient_account = self.kv_db.entry(&recipient_hash)?.or_insert_with(|| {
            tracing::info!("creating new account for recipient");
            Account::new(recipient.clone(), 0, 0)
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

    pub fn withdraw(
        &mut self,
        withdrawer: &PublicKey,
        withdraw: &Withdraw,
        nonce: u64,
    ) -> Result<(), AppErr> {
        tracing::info!("verifying the withdrawal can be applied");

        let [withdrawer_hash] = hash_buffers([withdrawer.as_slice()]);

        let withdrawer_account = self
            .kv_db
            .get(&withdrawer_hash)?
            .ok_or_else(|| anyhow!("withdrawer does not have an account"))?;

        withdrawer_account.check_nonce(nonce)?;

        if withdrawer_account.pubkey != *withdrawer {
            return Err(anyhow!("hash collision detected on withdrawer account").into());
        }

        if withdrawer_account.balance < withdraw.amount {
            return Err(anyhow!("withdrawer has insufficient funds").into());
        }

        let withdrawer_account = self.kv_db.entry(&withdrawer_hash)?.or_insert_with(|| {
            unreachable!("withdrawer account should exist");
        });

        withdrawer_account.balance = withdrawer_account
            .balance
            .checked_sub(withdraw.amount)
            .expect("sender balance underflow");

        withdrawer_account.increment_nonce();

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Account {
    pub pubkey: PublicKey,
    pub balance: u64,
    // Start at 0. Each transfer or withdrawal's `nonce` must match the account's `nonce`.
    // Each successful transfer or withdrawal increments the `nonce`.
    pub nonce: u64,
}

impl Account {
    pub fn new(pubkey: PublicKey, balance: u64, nonce: u64) -> Self {
        Self {
            pubkey,
            balance,
            nonce,
        }
    }

    pub fn check_nonce(&self, nonce: u64) -> Result<(), AppErr> {
        if self.nonce != nonce {
            return Err(anyhow!(
                "nonce mismatch: transaction nonce {nonce} does not match account nonce {}",
                self.nonce
            )
            .into());
        }

        Ok(())
    }

    pub fn increment_nonce(&mut self) {
        // TODO this is not a real problem an account will never have 2^64 transactions
        // To make nonce wrapping safe
        // we should expire transactions after a certain number of batches.
        self.nonce = self.nonce.wrapping_add(1);
    }
}

impl PortableHash for Account {
    fn portable_hash<H: PortableUpdate>(&self, hasher: &mut H) {
        self.pubkey.portable_hash(hasher);
        self.balance.portable_hash(hasher);
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
