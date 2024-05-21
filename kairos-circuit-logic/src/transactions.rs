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

#[cfg(feature = "arbitrary")]
pub use arbitrary_bounds::*;
#[cfg(feature = "arbitrary")]
mod arbitrary_bounds {
    use std::{cell::RefCell, collections::HashMap, fmt, ops::Deref, rc::Rc};

    use proptest::{prelude::*, sample};
    use test_strategy::Arbitrary;

    use super::*;
    use crate::account_trie::Account;

    #[derive(Debug, Clone)]
    pub enum TxnExpectedResult {
        Success,
        Failure,
    }

    #[derive(Debug, Clone, Arbitrary)]
    #[arbitrary(args = AccountsState)]
    pub struct RandomTransfer(
        #[strategy(any::<(sample::Index, sample::Index, sample::Index)>())]
        #[map(|(sender, recipient, amount)| args.random_transfer(sender, recipient, amount, 0.))]
        pub (Signed<Transfer>, TxnExpectedResult),
    );

    #[derive(Debug, Clone, Arbitrary)]
    #[arbitrary(args = AccountsState)]
    pub struct RandomWithdraw(
        #[strategy(any::<(sample::Index, sample::Index)>())]
        #[map(|(sender, amount)| args.random_withdraw(sender, amount))]
        pub (Signed<Withdraw>, TxnExpectedResult),
    );

    #[derive(Debug, Clone, Arbitrary)]
    #[arbitrary(args = AccountsState)]
    pub struct RandomL1Deposit(
        #[strategy(any::<(sample::Index, sample::Index, sample::Index)>())]
        #[map(|(sender, recipient, amount)| args.random_deposit(sender, recipient, amount))]
        pub (L1Deposit, TxnExpectedResult),
    );

    #[derive(Debug, Clone, Arbitrary)]
    #[arbitrary(args = AccountsState)]
    pub enum RandomTransaction {
        #[any(args)]
        Transfer(RandomTransfer),
        #[any(args)]
        Withdraw(RandomWithdraw),
        #[any(args)]
        L1Deposit(RandomL1Deposit),
    }

    #[derive(Debug, Clone, Arbitrary)]
    #[arbitrary(args = AccountsState)]
    pub struct ValidRandomTransaction(
        #[strategy(any_with::<RandomTransaction>(args.clone()))]
        #[filter(|txn| match txn {
            RandomTransaction::Transfer(RandomTransfer((_, TxnExpectedResult::Success))) |
            RandomTransaction::Withdraw(RandomWithdraw((_, TxnExpectedResult::Success))) |
            RandomTransaction::L1Deposit(RandomL1Deposit((_, TxnExpectedResult::Success))) => true,
            _ => false,
        })]
        pub RandomTransaction,
    );

    #[derive(Debug, Default)]
    pub struct PublicKeys(pub Vec<Rc<PublicKey>>);

    #[derive(Clone)]
    pub struct AccountsState {
        shared: Rc<AccountsStateInner>,
    }
    pub struct AccountsStateInner {
        pub keys: PublicKeys,
        pub l1_accounts: RefCell<HashMap<Rc<PublicKey>, u64>>,
        pub l2_accounts: RefCell<HashMap<Rc<PublicKey>, Account>>,
    }

    impl fmt::Debug for AccountsState {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(
                f,
                "\nAccountsState: {{\nAccountsState.l1_accounts: {:?}",
                self.shared.deref().l1_accounts
            )?;

            writeln!(
                f,
                "AccountsState.l2_accounts: {:?}\n}}",
                self.shared.l2_accounts
            )
        }
    }

    impl Default for AccountsState {
        fn default() -> Self {
            unreachable!("AccountsState should always be created with AccountsState::new()");
        }
    }

    impl AccountsState {
        pub fn new() -> Self {
            AccountsState {
                shared: Rc::new(AccountsStateInner {
                    keys: PublicKeys::default(),
                    l1_accounts: RefCell::new(HashMap::new()),
                    l2_accounts: RefCell::new(HashMap::new()),
                }),
            }
        }

        pub fn sample_keys(&self, sampler: sample::Index) -> PublicKey {
            let keys = &self.shared.keys;
            keys.0[sampler.index(keys.0.len())].to_vec()
        }

        pub fn random_deposit(
            &self,
            sender: sample::Index,
            recipient: sample::Index,
            amount_sampler: sample::Index,
        ) -> (L1Deposit, TxnExpectedResult) {
            let sender = self.sample_keys(sender);
            let recipient = self.sample_keys(recipient);

            let mut l1_accounts = self.shared.l1_accounts.borrow_mut();
            let l1_balance = l1_accounts
                .get_mut(&sender)
                .expect("sender does not have an l1 account in AccountsState");

            let amount = amount_sampler.index(*l1_balance as usize) as u64;

            let mut l2_accounts = self.shared.l2_accounts.borrow_mut();

            let l2_account = l2_accounts
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
            &self,
            sender: sample::Index,
            recipient: sample::Index,
            amount: sample::Index,
            insufficient_balance_prop: f64,
        ) -> (Signed<Transfer>, TxnExpectedResult) {
            let sender = self.sample_keys(sender);
            let recipient = self.sample_keys(recipient);

            let mut l2_accounts = self.shared.l2_accounts.borrow_mut();
            let sender_account = l2_accounts
                .get(&sender)
                .expect("sender does not have an l2 account in AccountsState");
            let sender_balance = sender_account.balance;
            let nonce = sender_account.nonce;

            let recipient_balance = l2_accounts
                .get(&recipient)
                .expect("recipient does not have an l2 account in AccountsState")
                .balance;

            // This not exact but is used to control the frequency of insufficient balance errors
            let amount = amount
                .index((sender_balance as f64 * (1. + insufficient_balance_prop)) as usize)
                as u64;

            let signed_transfer = |public_key: PublicKey, recipient: PublicKey| Signed {
                public_key,
                nonce,
                transaction: Transfer { recipient, amount },
            };

            match (
                sender_balance.checked_sub(amount),
                recipient_balance.checked_add(amount),
            ) {
                (Some(new_sender_bal), Some(new_recipient_bal)) => {
                    let sender_account = l2_accounts.get_mut(&sender).unwrap();
                    sender_account.balance = new_sender_bal;
                    sender_account.nonce += 1;

                    l2_accounts.get_mut(&recipient).unwrap().balance = new_recipient_bal;

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
            &self,
            sender: sample::Index,
            amount: sample::Index,
        ) -> (Signed<Withdraw>, TxnExpectedResult) {
            let sender = self.sample_keys(sender);

            let mut l2_accounts = self.shared.l2_accounts.borrow_mut();
            let sender_account = l2_accounts
                .get(&sender)
                .expect("sender does not have an l2 account in AccountsState");
            let sender_balance = sender_account.balance;
            let nonce = sender_account.nonce;

            let mut l1_accounts = self.shared.l1_accounts.borrow_mut();
            let l1_balance = l1_accounts
                .get_mut(&sender)
                .expect("recipient does not have an l2 account in AccountsState");

            // This not exact but is used to control the frequency of insufficient balance errors
            let insufficient_balance_prop = 0.10;
            let amount = amount
                .index((sender_balance as f64 * (1. + insufficient_balance_prop)) as usize)
                as u64;

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
                    let sender_account = l2_accounts.get_mut(&sender).unwrap();
                    sender_account.balance = new_sender_bal;
                    sender_account.nonce += 1;

                    *l1_balance = new_recipient_bal;

                    (signed_withdraw(sender), TxnExpectedResult::Success)
                }
                _ => (signed_withdraw(sender), TxnExpectedResult::Failure),
            }
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
