use alloc::vec::Vec;

#[cfg(feature = "asn1")]
use kairos_tx::{asn, error::TxError};

pub type PublicKey = Vec<u8>;

/// TODO remove this with future PR
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Transaction {
    Transfer(Transfer),
    Deposit(Deposit),
    Withdraw(Withdraw),
}
/// Transfer is between L2 accounts, entirely executed on L2.
/// Withdraw is initiated on L2 and executed on the L1.
/// Deposit comes from the L1, and is executed first on L1 and then on L2.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KairosTransaction {
    Transfer(Signed<Transfer>),
    Withdraw(Signed<Withdraw>),
    Deposit(L1Deposit),
}

/// A signed transaction.
/// The signature should already be verified before you construct this type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Signed<T> {
    pub public_key: PublicKey,
    /// Increments with each Transfer or Withdraw from this account.
    pub nonce: u64,
    pub transaction: T,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Transfer {
    pub recipient: PublicKey,
    pub amount: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct L1Deposit {
    pub recipient: PublicKey,

    pub amount: u64,
}

/// TODO remove this struct
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Deposit {
    pub amount: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Withdraw {
    pub amount: u64,
}

#[cfg(any(test, feature = "arbitrary"))]
pub mod arbitrary {
    use core::{fmt::Debug, ops::RangeInclusive};
    use std::{collections::HashMap, fmt, ops::Deref, rc::Rc};

    use kairos_trie::{stored::memory_db::MemoryDb, DigestHasher, KeyHash, NodeHash, TrieRoot};
    use proptest::{collection as prop, prelude::*, sample};
    use sha2::{digest::FixedOutputReset, Digest, Sha256};
    use test_strategy::Arbitrary;

    use super::*;
    use crate::account_trie::{Account, AccountTrie};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum TxnExpectedResult {
        Success,
        Failure,
    }

    #[derive(Debug, Clone)]
    pub struct RandomBatchesConfig {
        pub batch_size: RangeInclusive<usize>,
        pub batch_count: RangeInclusive<usize>,
        pub initial_l1_accounts: RangeInclusive<usize>,
        pub initial_l2_accounts: RangeInclusive<usize>,
    }

    impl Default for RandomBatchesConfig {
        fn default() -> Self {
            Self {
                batch_size: 1..=100,
                batch_count: 1..=10,
                initial_l1_accounts: 1..=10,
                initial_l2_accounts: 1..=1000,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct RandomBatches {
        pub initial_trie: TrieRoot<NodeHash>,
        pub trie_db: Rc<MemoryDb<Account>>,
        pub initial_state: AccountsState,
        pub final_state: AccountsState,
        pub batches: Vec<Vec<(KairosTransaction, TxnExpectedResult)>>,
    }

    impl RandomBatches {
        pub fn filter_success(&self) -> Vec<Vec<KairosTransaction>> {
            self.batches
                .iter()
                .map(|batch| {
                    batch
                        .iter()
                        .filter_map(|(txn, res)| {
                            if *res == TxnExpectedResult::Success {
                                Some(txn.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .filter(|batch| !batch.is_empty())
                .collect()
        }
    }

    impl Arbitrary for RandomBatches {
        type Parameters = RandomBatchesConfig;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(config: Self::Parameters) -> Self::Strategy {
            AccountsState::arbitrary_with(config.clone())
                .prop_flat_map(move |initial_state| {
                    prop::vec(
                        prop::vec(
                            (0..3, any::<(sample::Index, sample::Index, sample::Index)>()),
                            config.batch_size.clone(),
                        ),
                        config.batch_count.clone(),
                    )
                    .prop_map(move |seed| {
                        let mut accounts_state = initial_state.clone();

                        let batches = seed
                            .into_iter()
                            .map(|batch| {
                                batch
                                    .into_iter()
                                    .map(|(kind, (sender, recipient, amount))| match kind {
                                        0 => {
                                            let (txn, res) = accounts_state
                                                .random_transfer(sender, recipient, amount);
                                            (KairosTransaction::Transfer(txn), res)
                                        }
                                        1 => {
                                            let (txn, res) =
                                                accounts_state.random_withdraw(sender, amount);
                                            (KairosTransaction::Withdraw(txn), res)
                                        }
                                        2 => {
                                            let (txn, res) = accounts_state
                                                .random_deposit(sender, recipient, amount);
                                            (KairosTransaction::Deposit(txn), res)
                                        }
                                        _ => unreachable!(),
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .collect::<Vec<_>>();

                        let (initial_trie, trie_db) = initial_state.build_trie();
                        Self {
                            batches,
                            initial_trie,
                            trie_db,
                            initial_state: initial_state.clone(),
                            final_state: accounts_state,
                        }
                    })
                    .boxed()
                })
                .boxed()
        }
    }

    /// A test model for the state of the accounts on both L1 and L2.
    /// This is used to generate random valid transactions.
    #[derive(Clone, Arbitrary)]
    #[arbitrary(args = RandomBatchesConfig)]
    pub struct AccountsState {
        #[any(args.deref().clone())]
        pub l1: Accounts<u64>,
        #[any(args.deref().clone())]
        pub l2: Accounts<Account>,
    }

    impl fmt::Debug for AccountsState {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(
                f,
                "\nAccountsState: {{\nAccountsState.l1.accounts: {:?}",
                self.l1.accounts
            )?;

            writeln!(f, "AccountsState.l2.accounts: {:?}\n}}", self.l2.accounts)
        }
    }

    impl Default for AccountsState {
        fn default() -> Self {
            unreachable!("AccountsState should always be created with AccountsState::new()");
        }
    }

    impl AccountsState {
        pub fn build_trie(&self) -> (TrieRoot<NodeHash>, Rc<MemoryDb<Account>>) {
            let mut account_trie =
                AccountTrie::new_try_from_db(Rc::new(MemoryDb::empty()), TrieRoot::Empty).unwrap();

            let mut hasher = Sha256::new();
            for (public_key, account) in self.l2.accounts.iter() {
                hasher.update(public_key.as_slice());
                let key = &KeyHash::from_bytes(&hasher.finalize_fixed_reset().into());

                account_trie.txn.insert(key, account.clone()).unwrap();
            }

            let root = account_trie.txn.commit(&mut DigestHasher(hasher)).unwrap();

            let db = account_trie.txn.data_store.db().clone();

            (root, db)
        }

        pub fn random_deposit(
            &mut self,
            sender: sample::Index,
            recipient: sample::Index,
            amount_sampler: sample::Index,
        ) -> (L1Deposit, TxnExpectedResult) {
            let sender = self.l1.sample_keys(sender);
            let recipient = self.l2.sample_keys(recipient).deref().clone();

            let l1_balance = self
                .l1
                .accounts
                .get_mut(&sender)
                .expect("sender does not have an l1 account in AccountsState");

            let amount = if *l1_balance == 0 {
                return (
                    L1Deposit {
                        recipient,
                        amount: 0,
                    },
                    TxnExpectedResult::Failure,
                );
            } else {
                amount_sampler.index(*l1_balance as usize) as u64 + 1
            };

            if *sender == recipient {
                return (L1Deposit { recipient, amount }, TxnExpectedResult::Failure);
            }

            let l2_account = self
                .l2
                .accounts
                .get_mut(&recipient)
                .expect("recipient does not have an l2 account in AccountsState");

            // if the deposit will fail don't change the test model state
            match (
                l1_balance.checked_sub(amount),
                l2_account.balance.checked_add(amount),
            ) {
                (Some(l1_bal), Some(l2_bal)) => {
                    *l1_balance = l1_bal;
                    l2_account.balance = l2_bal;

                    (L1Deposit { recipient, amount }, TxnExpectedResult::Success)
                }
                _ => {
                    unreachable!("For now I am not testing the case where the deposit fails");
                    // (L1Deposit { recipient, amount }, TxnExpectedResult::Failure)
                }
            }
        }

        pub fn random_transfer(
            &mut self,
            sender: sample::Index,
            recipient: sample::Index,
            amount: sample::Index,
        ) -> (Signed<Transfer>, TxnExpectedResult) {
            let sender = self.l2.sample_keys(sender).deref().clone();
            let recipient = self.l2.sample_keys(recipient).deref().clone();

            let sender_account = self
                .l2
                .accounts
                .get(&sender)
                .expect("sender does not have an l2 account in AccountsState");
            let sender_balance = sender_account.balance;
            let nonce = sender_account.nonce;

            let recipient_balance = self
                .l2
                .accounts
                .get(&recipient)
                .expect("recipient does not have an l2 account in AccountsState")
                .balance;

            let amount = if sender_balance == 0 {
                return (
                    Signed {
                        public_key: sender.clone(),
                        nonce,
                        transaction: Transfer {
                            recipient: recipient.clone(),
                            amount: 0,
                        },
                    },
                    TxnExpectedResult::Failure,
                );
            } else {
                amount.index(sender_balance as usize) as u64 + 1
            };

            let signed_transfer = |public_key: PublicKey, recipient: PublicKey| Signed {
                public_key,
                nonce,
                transaction: Transfer { recipient, amount },
            };

            match (
                sender_balance.checked_sub(amount),
                recipient_balance.checked_add(amount),
            ) {
                (Some(_), Some(_)) => {
                    let sender_account = self.l2.accounts.get_mut(&sender).unwrap();
                    sender_account.balance -= amount;
                    sender_account.nonce += 1;

                    self.l2.accounts.get_mut(&recipient).unwrap().balance += amount;

                    (
                        signed_transfer(sender, recipient),
                        TxnExpectedResult::Success,
                    )
                }
                _ => (
                    signed_transfer(sender, recipient),
                    TxnExpectedResult::Failure,
                ),
            }
        }

        pub fn random_withdraw(
            &mut self,
            sender: sample::Index,
            amount: sample::Index,
        ) -> (Signed<Withdraw>, TxnExpectedResult) {
            let sender = self.l2.sample_keys(sender);

            let sender_account = self
                .l2
                .accounts
                .get(&sender)
                .expect("sender does not have an l2 account in AccountsState");
            let sender_balance = sender_account.balance;
            let nonce = sender_account.nonce;

            let l1_balance = self.l1.get_mut_or_insert_with(sender.clone(), || 0);

            let amount = if sender_balance == 0 {
                return (
                    Signed {
                        public_key: sender.deref().clone(),
                        nonce,
                        transaction: Withdraw { amount: 0 },
                    },
                    TxnExpectedResult::Failure,
                );
            } else {
                amount.index(sender_balance as usize) as u64 + 1
            };

            let signed_withdraw = |public_key: PublicKey| Signed {
                public_key,
                nonce,
                transaction: Withdraw { amount },
            };

            match (
                sender_balance.checked_sub(amount),
                l1_balance.checked_add(amount),
            ) {
                (Some(new_sender_bal), Some(new_recipient_bal)) => {
                    let sender_account = self.l2.accounts.get_mut(&sender).unwrap();
                    sender_account.balance = new_sender_bal;
                    sender_account.nonce += 1;

                    *l1_balance = new_recipient_bal;

                    (signed_withdraw(sender.to_vec()), TxnExpectedResult::Success)
                }
                _ => (signed_withdraw(sender.to_vec()), TxnExpectedResult::Failure),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Accounts<A: Debug> {
        pub pub_keys: Vec<Rc<PublicKey>>,
        pub accounts: HashMap<Rc<PublicKey>, A>,
    }

    impl<A: Debug> Default for Accounts<A> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<A: Debug> Accounts<A> {
        pub fn new() -> Self {
            Accounts {
                pub_keys: Vec::new(),
                accounts: HashMap::new(),
            }
        }

        pub fn sample_keys(&self, sampler: sample::Index) -> Rc<PublicKey> {
            self.pub_keys[sampler.index(self.len())].clone()
        }

        pub fn len(&self) -> usize {
            self.accounts.len()
        }

        pub fn is_empty(&self) -> bool {
            self.accounts.is_empty()
        }

        pub fn get_mut_or_insert_with(
            &mut self,
            key: Rc<PublicKey>,
            default: impl FnOnce() -> A,
        ) -> &mut A {
            self.pub_keys.push(key.clone());
            self.accounts.entry(key).or_insert_with(default)
        }
    }

    impl Arbitrary for Accounts<u64> {
        type Parameters = RandomBatchesConfig;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(config: Self::Parameters) -> Self::Strategy {
            (prop::hash_map(
                any::<[u8; 32]>().prop_map(|pk| Rc::new(pk.into())),
                1..100_000u64,
                config.initial_l1_accounts.clone(),
            ))
            .prop_map(|accounts| Accounts {
                pub_keys: accounts.keys().cloned().collect(),
                accounts,
            })
            .boxed()
        }
    }

    impl Arbitrary for Accounts<Account> {
        type Parameters = RandomBatchesConfig;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(config: Self::Parameters) -> Self::Strategy {
            (prop::hash_map(
                any::<[u8; 32]>().prop_map(|pk| Rc::new(pk.into())),
                (1..10u64, 0..100u64),
                config.initial_l2_accounts.clone(),
            ))
            .prop_map(|accounts| Accounts {
                pub_keys: accounts.keys().cloned().collect(),
                accounts: accounts
                    .into_iter()
                    .map(|(public_key, (balance, nonce))| {
                        let pubkey = public_key.deref().clone();
                        (
                            public_key,
                            Account {
                                pubkey,
                                balance,
                                nonce,
                            },
                        )
                    })
                    .collect(),
            })
            .boxed()
        }
    }
}

#[cfg(feature = "asn1")]
impl TryFrom<asn::Transfer> for Transfer {
    type Error = TxError;
    fn try_from(transfer: asn::Transfer) -> Result<Self, Self::Error> {
        Ok(Transfer {
            recipient: transfer.recipient.into(),
            amount: transfer.amount.try_into()?,
        })
    }
}

#[cfg(feature = "asn1")]
impl TryFrom<asn::Deposit> for Deposit {
    type Error = TxError;
    fn try_from(deposit: asn::Deposit) -> Result<Self, Self::Error> {
        Ok(Deposit {
            amount: deposit.amount.try_into()?,
        })
    }
}

#[cfg(feature = "asn1")]
impl TryFrom<asn::Withdrawal> for Withdraw {
    type Error = TxError;
    fn try_from(withdrawal: asn::Withdrawal) -> Result<Self, Self::Error> {
        Ok(Withdraw {
            amount: withdrawal.amount.try_into()?,
        })
    }
}
