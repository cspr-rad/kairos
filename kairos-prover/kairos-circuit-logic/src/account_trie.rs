use alloc::{boxed::Box, format, string::String, vec::Vec};

use sha2::{digest::FixedOutputReset, Digest, Sha256};

use crate::transactions::{L1Deposit, L2Transactions, PublicKey, Signed, Transfer, Withdraw};
use kairos_trie::{stored::merkle::Snapshot, KeyHash, PortableHash, PortableUpdate};

/// The state of the batch transaction against the trie.
pub struct AccountTrie<'s> {
    pub txn: AccountTrieTxn<'s>,
}
pub type AccountTrieTxn<'s> = kairos_trie::Transaction<&'s Snapshot<Account>, Account>;

/// TODO panic on error should be behind a feature flag
type TxnErr = String;

impl<'s> TryFrom<&'s Snapshot<Account>> for AccountTrie<'s> {
    type Error = TxnErr;

    fn try_from(snapshot: &'s Snapshot<Account>) -> Result<Self, Self::Error> {
        Self::new_try_from_snapshot(snapshot)
    }
}

impl<'s> AccountTrie<'s> {
    pub fn new_try_from_snapshot(snapshot: &'s Snapshot<Account>) -> Result<Self, TxnErr> {
        Ok(Self {
            txn: kairos_trie::Transaction::from_snapshot(snapshot)?,
        })
    }

    pub fn apply_batch(
        &mut self,
        l1_deposits: &[L1Deposit],
        l2_transactions: impl Iterator<Item = Signed<L2Transactions>>,
    ) -> Result<Box<[Signed<Withdraw>]>, TxnErr> {
        for deposit in l1_deposits {
            self.deposit(deposit.clone())?;
        }

        let mut l2_withdrawals = Vec::new();
        for signed_txn in l2_transactions {
            match &signed_txn.transaction {
                L2Transactions::Transfer(transfer) => {
                    self.transfer(&signed_txn.public_key, transfer, signed_txn.nonce)?;
                }
                L2Transactions::Withdraw(withdraw) => {
                    self.withdraw(&signed_txn.public_key, withdraw, signed_txn.nonce)?;
                    l2_withdrawals.push(Signed {
                        public_key: signed_txn.public_key,
                        nonce: signed_txn.nonce,
                        transaction: Withdraw {
                            amount: withdraw.amount,
                        },
                    });
                }
            }
        }

        Ok(l2_withdrawals.into_boxed_slice())
    }

    /// Avoid calling this method with a transfer that will fail.
    /// This method's prechecks may cause the Snapshot to bloat with unnecessary data if the transfer fails.
    pub fn transfer(
        &mut self,
        sender: &PublicKey,
        transfer: &Transfer,
        nonce: u64,
    ) -> Result<(), TxnErr> {
        let [sender_hash, recipient_hash] =
            hash_buffers([sender.as_slice(), transfer.recipient.as_slice()]);

        let mut sender_account = self.txn.entry(&sender_hash)?;

        let sender_account = sender_account
            .get_mut()
            .ok_or("sender does not have an account")?;

        if sender_account.pubkey != *sender {
            return Err(("hash collision detected on sender account").into());
        }

        sender_account.check_nonce(nonce)?;
        sender_account.increment_nonce();

        // Remove the amount from the sender's account
        sender_account.balance = sender_account
            .balance
            .checked_sub(transfer.amount)
            .ok_or("Sender balance underflow")?;

        let recipient_account = self
            .txn
            .entry(&recipient_hash)?
            .or_insert_with(|| Account::new(transfer.recipient.clone(), 0, 0));

        if recipient_account.pubkey != transfer.recipient {
            return Err(("hash collision detected on recipient account").into());
        }

        // Add the amount to the recipient's account
        recipient_account.balance = recipient_account
            .balance
            .checked_add(transfer.amount)
            .ok_or("recipient balance overflow")?;

        Ok(())
    }

    pub fn deposit(&mut self, deposit: L1Deposit) -> Result<(), TxnErr> {
        let [recipient_hash] = hash_buffers([deposit.recipient.as_slice()]);

        let mut recipient_account = self.txn.entry(&recipient_hash)?;

        let recipient_account = if let Some(recipient_account) = recipient_account.get_mut() {
            if recipient_account.pubkey != deposit.recipient {
                return Err(("hash collision detected on recipient account").into());
            }

            recipient_account
        } else {
            recipient_account.or_insert_with(|| Account::new(deposit.recipient, 0, 0))
        };

        recipient_account.balance = recipient_account
            .balance
            .checked_add(deposit.amount)
            .ok_or("recipient balance overflow")?;

        Ok(())
    }

    pub fn withdraw(
        &mut self,
        withdrawer: &PublicKey,
        withdraw: &Withdraw,
        nonce: u64,
    ) -> Result<(), TxnErr> {
        let [withdrawer_hash] = hash_buffers([withdrawer.as_slice()]);

        let mut withdrawer_account = self.txn.entry(&withdrawer_hash)?;

        let withdrawer_account = withdrawer_account
            .get_mut()
            .ok_or("withdrawer does not have an account")?;

        if withdrawer_account.pubkey != *withdrawer {
            return Err(("hash collision detected on withdrawer account").into());
        }

        withdrawer_account.check_nonce(nonce)?;
        withdrawer_account.increment_nonce();

        withdrawer_account.balance = withdrawer_account
            .balance
            .checked_sub(withdraw.amount)
            .ok_or("sender balance underflow")?;

        Ok(())
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    pub fn check_nonce(&self, nonce: u64) -> Result<(), TxnErr> {
        if self.nonce != nonce {
            return Err(format!(
                "nonce mismatch: transaction nonce {nonce} does not match account nonce {}",
                self.nonce,
            ));
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
