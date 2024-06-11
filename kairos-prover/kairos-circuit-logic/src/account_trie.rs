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
    pub fn new_try_from_db(db: Db, root_hash: TrieRoot<NodeHash>) -> Self {
        Self {
            txn: kairos_trie::Transaction::from_snapshot_builder(
                SnapshotBuilder::empty(db).with_trie_root_hash(root_hash),
            ),
        }
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

    /// If `transfer` fails the `Transaction` may contain torn writes to the trie.
    /// You must throw away the `Transaction` if it `transfer` fails.
    pub fn transfer(
        &mut self,
        sender: &PublicKey,
        transfer: &Transfer,
        nonce: u64,
    ) -> Result<(), TxnErr> {
        if sender == &transfer.recipient {
            return Err("Transfer Failed: sender and recipient are the same".into());
        }

        let [sender_hash, recipient_hash] =
            hash_buffers([sender.as_slice(), transfer.recipient.as_slice()]);

        let mut sender_account = self.txn.entry(&sender_hash)?;

        let sender_account = sender_account.get_mut().ok_or_else(|| {
            format!(
                "sender does not have an account, sender: `{sender:?}`, sender_hash: `{sender_hash:?}`"
            )
        })?;

        // SECURITY ASUMPTION: see Account docs
        // if sender_account.public_key != *sender {
        //     return Err(("hash collision detected on sender account").into());
        // }

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
            .or_insert_with(|| Account::new(0, 0));

        // SECURITY ASUMPTION: see Account docs
        // if recipient_account.public_key != transfer.recipient {
        //     return Err(("hash collision detected on recipient account").into());
        // }

        // Add the amount to the recipient's account
        recipient_account.balance = recipient_account
            .balance
            .checked_add(transfer.amount)
            .ok_or("recipient balance overflow")?;

        Ok(())
    }

    /// If `deposit` fails the `Transaction` may contain torn writes to the trie.
    /// You should throw away the `Transaction` if it `deposit` fails.
    pub fn deposit(&mut self, deposit: &L1Deposit) -> Result<(), TxnErr> {
        let [recipient_hash] = hash_buffers([deposit.recipient.as_slice()]);

        // SECURITY ASUMPTION: see Account docs
        // if recipient_account.public_key != deposit.recipient {
        //     return Err(("hash collision detected on recipient account").into());
        // }

        let recipient_account = self
            .txn
            .entry(&recipient_hash)?
            .or_insert_with(|| Account::new(0, 0));

        recipient_account.balance = recipient_account
            .balance
            .checked_add(deposit.amount)
            .ok_or("Deposit Failed: recipient balance overflow")?;

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
            .ok_or("Withdraw Failed: withdrawer does not have an account")?;

        // SECURITY ASUMPTION: see Account docs
        // if withdrawer_account.public_key != *withdrawer {
        //     return Err(("hash collision detected on withdrawer account").into());
        // }

        withdrawer_account.check_nonce(nonce)?;
        withdrawer_account.increment_nonce();

        withdrawer_account.balance = withdrawer_account
            .balance
            .checked_sub(withdraw.amount)
            .ok_or("Withdraw Failed: withdrawer has insufficient funds")?;

        Ok(())
    }
}

impl<Db: DatabaseGet<Account>> AccountTrie<SnapshotBuilder<Db, Account>> {
    /// Check the preconditions for a transfer.
    /// This method should only be used on the server when building `Snapshot`.
    ///
    /// Prechecking prevents the batch `Transaction` from containing torn writes
    /// and Snapshot from bloating with unnecessary data.
    ///
    /// The `transfer` method also checks all of these conditions, excluding `amount == 0`.
    /// However if `transfer` fails the `Transaction` may contain torn writes to the trie.
    /// You must throw away the `Transaction` if it `transfer` fails.
    pub fn precheck_transfer(
        &self,
        sender: &PublicKey,
        transfer: &Transfer,
        nonce: u64,
    ) -> Result<(), TxnErr> {
        if transfer.amount == 0 {
            return Err("Transfer Failed: transfer amount is zero".into());
        }

        if sender == &transfer.recipient {
            return Err("Transfer Failed: sender and recipient are the same".into());
        }

        let [sender_hash, recipient_hash] =
            hash_buffers([sender.as_slice(), transfer.recipient.as_slice()]);

        let sender_account = self
            .txn
            .get_exclude_from_txn(&sender_hash)?
            .ok_or("Transfer Failed: sender does not have an account")?;

        // SECURITY ASUMPTION: see Account docs
        // if sender_account.public_key != *sender {
        //     return Err(("hash collision detected on sender account").into());
        // }

        sender_account.check_nonce(nonce)?;

        if sender_account.balance < transfer.amount {
            return Err("Transfer Failed: sender has insufficient funds".into());
        }

        if let Some(recipient_account) = self.txn.get_exclude_from_txn(&recipient_hash)? {
            recipient_account
                .balance
                .checked_add(transfer.amount)
                .ok_or("Transfer Failed: recipient balance overflow")?;
        }

        Ok(())
    }

    /// Check the preconditions for a deposit.
    /// This method should only be used on the server when building `Snapshot`.
    pub fn precheck_deposit(&self, deposit: &L1Deposit) -> Result<(), TxnErr> {
        if deposit.amount == 0 {
            return Err("Deposit Failed: deposit amount is zero".into());
        }

        let [recipient_hash] = hash_buffers([deposit.recipient.as_slice()]);

        // SECURITY ASUMPTION: see Account docs
        // if recipient_account.public_key != deposit.recipient {
        //     return Err(("hash collision detected on recipient account").into());
        // }

        if let Some(recipient_account) = self.txn.get_exclude_from_txn(&recipient_hash)? {
            recipient_account
                .balance
                .checked_add(deposit.amount)
                .ok_or("Deposit Failed: recipient balance overflow")?;
        }

        Ok(())
    }

    /// check the preconditions for a withdraw.
    /// this method should only be used on the server when building `snapshot`.
    pub fn precheck_withdraw(
        &self,
        withdrawer: &PublicKey,
        withdraw: &Withdraw,
        nonce: u64,
    ) -> Result<(), TxnErr> {
        if withdraw.amount == 0 {
            return Err("Withdraw Failed: withdraw amount is zero".into());
        }

        let [withdrawer_hash] = hash_buffers([withdrawer.as_slice()]);

        let withdrawer_account = self
            .txn
            .get_exclude_from_txn(&withdrawer_hash)?
            .ok_or("Withdraw Failed: withdrawer does not have an account")?;

        // SECURITY ASUMPTION: see Account docs
        // if withdrawer_account.public_key != *withdrawer {
        //     return Err(("hash collision detected on withdrawer account").into());
        // }

        withdrawer_account.check_nonce(nonce)?;

        if withdrawer_account.balance < withdraw.amount {
            Err("Withdraw Failed: withdrawer has insufficient funds".into())
        } else {
            Ok(())
        }
    }
}

/// An account in the trie.
/// Stores the balance and nonce owned by a public key.
///
/// SECURITY ASUMPTION: We assume that the sha256(public_key) will not collide with another public_key.
/// We do not check the preimage of the hash is the same public_key that created the account.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Account {
    // pub public_key: PublicKey,
    pub balance: u64,
    /// Start at 0. Each transfer or withdrawal's `nonce` must match the account's `nonce`.
    /// Each successful transfer or withdrawal increments the `nonce`.
    pub nonce: u64,
}

impl Account {
    pub fn new(balance: u64, nonce: u64) -> Self {
        Self { balance, nonce }
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
        let Self { balance, nonce } = self;
        balance.portable_hash(hasher);
        nonce.portable_hash(hasher);
    }
}

/// A utility function to hash multiple buffers reusing the same hasher.
/// Note this function returns an array of hashes, one for each input item.
/// `out[i] = hash(item[i])`
fn hash_buffers<const N: usize, T: AsRef<[u8]>>(items: [T; N]) -> [KeyHash; N] {
    let mut out = [KeyHash([0; 8]); N];
    let mut hasher = Sha256::new();

    for (i, item) in items.iter().enumerate() {
        hasher.update(item.as_ref());
        out[i] = KeyHash::from_bytes(&hasher.finalize_fixed_reset().into());
    }

    out
}

#[cfg(any(test, feature = "test-logic"))]
pub mod test_logic {
    use crate::{ProofInputs, ProofOutputs};

    use super::*;
    use alloc::rc::Rc;
    use kairos_trie::{stored::memory_db::MemoryDb, DigestHasher};

    pub fn test_prove_batch(
        mut prior_root_hash: TrieRoot<NodeHash>,
        db: Rc<MemoryDb<Account>>,
        batches: Vec<Vec<KairosTransaction>>,
        proving_hook: impl Fn(ProofInputs) -> Result<ProofOutputs, String>,
    ) {
        for batch in batches.into_iter() {
            let mut account_trie = AccountTrie::new_try_from_db(db.clone(), prior_root_hash);
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

            let pre_batch_trie_root: TrieRoot<NodeHash> = pre_batch_trie_root.into();
            let post_batch_trie_root: TrieRoot<NodeHash> = post_batch_trie_root.into();

            assert_eq!(pre_batch_trie_root, prior_root_hash);
            assert_eq!(post_batch_trie_root, new_root_hash);
            prior_root_hash = new_root_hash;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::transactions::arbitrary::RandomBatches;

        use proptest::prelude::*;

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

            test_prove_batch(
                TrieRoot::Empty,
                Rc::new(MemoryDb::<Account>::empty()),
                batches,
                |proof_inputs| proof_inputs.run_batch_proof_logic(),
            )
        }

        #[test_strategy::proptest(ProptestConfig::default(), cases = 50)]
        fn proptest_prove_batches(
            #[any(batch_size = 1..=1000, batch_count = 2..=10)] args: RandomBatches,
        ) {
            let batches = args.filter_success();

            proptest::prop_assume!(batches.len() >= 2);

            test_prove_batch(args.initial_trie, args.trie_db, batches, |proof_inputs| {
                proof_inputs.run_batch_proof_logic()
            })
        }
    }
}
