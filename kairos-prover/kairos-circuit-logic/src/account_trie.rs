use alloc::{boxed::Box, format, string::String, vec::Vec};

use sha2::{digest::FixedOutputReset, Digest, Sha256};

use crate::transactions::{KairosTransaction, L1Deposit, PublicKey, Signed, Transfer, Withdraw};
use kairos_trie::{
    stored::{
        merkle::{Snapshot, SnapshotBuilder},
        DatabaseGet, Store,
    },
    KeyHash, NodeHash, PortableHash, PortableUpdate, TrieRoot,
};

/// The state of the batch transaction against the trie.
pub struct AccountTrie<S: Store<Account>> {
    pub txn: AccountTrieTxn<S>,
}
pub type AccountTrieTxn<S> = kairos_trie::Transaction<S, Account>;

/// TODO panic on error should be behind a feature flag
type TxnErr = String;

impl<'s> TryFrom<&'s Snapshot<Account>> for AccountTrie<&'s Snapshot<Account>> {
    type Error = TxnErr;

    fn try_from(snapshot: &'s Snapshot<Account>) -> Result<Self, Self::Error> {
        Ok(Self {
            txn: kairos_trie::Transaction::from_snapshot(snapshot)?,
        })
    }
}

impl<Db: 'static + DatabaseGet<Account>> TryFrom<SnapshotBuilder<Db, Account>>
    for AccountTrie<SnapshotBuilder<Db, Account>>
{
    type Error = TxnErr;
    fn try_from(snapshot: SnapshotBuilder<Db, Account>) -> Result<Self, Self::Error> {
        Ok(Self {
            txn: kairos_trie::Transaction::from_snapshot_builder(snapshot),
        })
    }
}

impl<'s> AccountTrie<&'s Snapshot<Account>> {
    pub fn new_try_from_snapshot(snapshot: &'s Snapshot<Account>) -> Result<Self, TxnErr> {
        Ok(Self {
            txn: kairos_trie::Transaction::from_snapshot(snapshot)?,
        })
    }
}

impl<Db: 'static + DatabaseGet<Account>> AccountTrie<SnapshotBuilder<Db, Account>> {
    pub fn new_try_from_db(db: Db, root_hash: TrieRoot<NodeHash>) -> Result<Self, TxnErr> {
        Ok(Self {
            txn: kairos_trie::Transaction::from_snapshot_builder(
                SnapshotBuilder::empty(db).with_trie_root_hash(root_hash),
            ),
        })
    }
}

impl<S: Store<Account>> AccountTrie<S> {
    #[allow(clippy::type_complexity)]
    pub fn apply_batch(
        &mut self,
        transactions: impl Iterator<Item = KairosTransaction>,
    ) -> Result<(Box<[L1Deposit]>, Box<[Signed<Withdraw>]>), TxnErr> {
        let mut l1_deposits = Vec::new();
        let mut l2_withdrawals = Vec::new();

        for txn in transactions {
            match txn {
                KairosTransaction::Transfer(transfer) => {
                    self.transfer(&transfer.public_key, &transfer.transaction, transfer.nonce)?;
                }
                KairosTransaction::Withdraw(withdraw) => {
                    self.withdraw(&withdraw.public_key, &withdraw.transaction, withdraw.nonce)?;
                    l2_withdrawals.push(withdraw);
                }

                KairosTransaction::Deposit(deposit) => {
                    self.deposit(&deposit)?;
                    l1_deposits.push(deposit);
                }
            }
        }

        Ok((
            l1_deposits.into_boxed_slice(),
            l2_withdrawals.into_boxed_slice(),
        ))
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

    pub fn deposit(&mut self, deposit: &L1Deposit) -> Result<(), TxnErr> {
        let [recipient_hash] = hash_buffers([deposit.recipient.as_slice()]);

        let mut recipient_account = self.txn.entry(&recipient_hash)?;

        let recipient_account = if let Some(recipient_account) = recipient_account.get_mut() {
            if recipient_account.pubkey != deposit.recipient {
                return Err(("hash collision detected on recipient account").into());
            }

            recipient_account
        } else {
            recipient_account.or_insert_with(|| Account::new(deposit.recipient.clone(), 0, 0))
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

#[cfg(feature = "test-logic")]
pub mod test_logic {
    use crate::{ProofInputs, ProofOutputs};

    use super::*;
    use alloc::rc::Rc;
    use kairos_trie::{stored::memory_db::MemoryDb, DigestHasher};

    pub fn test_prove_batch(
        batches: Vec<Vec<KairosTransaction>>,
        proving_hook: impl Fn(ProofInputs) -> Result<ProofOutputs, String>,
    ) {
        let db = Rc::new(MemoryDb::<Account>::empty());
        let mut prior_root_hash = TrieRoot::default();

        for batch in batches.into_iter() {
            let mut account_trie = AccountTrie::new_try_from_db(db.clone(), prior_root_hash)
                .expect("Failed to create account trie");
            account_trie
                .apply_batch(batch.iter().cloned())
                .expect("Failed to apply batch");

            let new_root_hash = account_trie
                .txn
                .commit(&mut DigestHasher::<sha2::Sha256>::default())
                .expect("Failed to commit transaction");

            let trie_snapshot = account_trie.txn.build_initial_snapshot();

            let proof_inputs = ProofInputs {
                transactions: batch.into_boxed_slice(),
                trie_snapshot,
            };

            let ProofOutputs {
                pre_batch_trie_root,
                post_batch_trie_root,
                deposits: _,
                withdrawals: _,
            } = proving_hook(proof_inputs).expect("Failed to prove execution");

            assert_eq!(pre_batch_trie_root, prior_root_hash);
            assert_eq!(post_batch_trie_root, new_root_hash);
            prior_root_hash = new_root_hash;
        }
    }

    #[test]
    fn test_prove_simple_batches() {
        let alice_public_key = "alice_public_key".as_bytes().to_vec();
        let bob_public_key = "bob_public_key".as_bytes().to_vec();

        let batches = vec![
            vec![
                KairosTransaction::Deposit(L1Deposit {
                    recipient: alice_public_key.clone(),
                    amount: 10,
                }),
                KairosTransaction::Transfer(Signed {
                    public_key: alice_public_key.clone(),
                    transaction: Transfer {
                        recipient: bob_public_key.clone(),
                        amount: 5,
                    },
                    nonce: 0,
                }),
                KairosTransaction::Withdraw(Signed {
                    public_key: alice_public_key.clone(),
                    transaction: Withdraw { amount: 5 },
                    nonce: 1,
                }),
            ],
            vec![
                KairosTransaction::Transfer(Signed {
                    public_key: bob_public_key.clone(),
                    transaction: Transfer {
                        recipient: alice_public_key.clone(),
                        amount: 2,
                    },
                    nonce: 0,
                }),
                KairosTransaction::Withdraw(Signed {
                    public_key: bob_public_key.clone(),
                    transaction: Withdraw { amount: 3 },
                    nonce: 1,
                }),
                KairosTransaction::Withdraw(Signed {
                    public_key: alice_public_key.clone(),
                    transaction: Withdraw { amount: 2 },
                    nonce: 2,
                }),
            ],
        ];

        test_prove_batch(batches, |proof_inputs| proof_inputs.run_batch_proof_logic())
    }
}
