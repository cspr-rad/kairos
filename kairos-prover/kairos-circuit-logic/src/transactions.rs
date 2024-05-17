use alloc::vec::Vec;

#[cfg(feature = "asn1")]
use kairos_tx::{asn, error::TxError};

pub type PublicKey = Vec<u8>;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Transaction {
    Transfer(Transfer),
    Deposit(Deposit),
    Withdraw(Withdraw),
}

/// These are the transactions that are initiated by the L2.
/// Deposit comes from the L1, Withdraw goes to the L1.
/// Transfer is between L2 accounts.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "arbitrary", derive(test_strategy::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum L2Transactions {
    Transfer(Transfer),
    // #[cfg_attr(feature = "arbitrary", weight(3))]
    Withdraw(Withdraw),
}

/// A signed transaction.
/// The signature should already be verified before yout construct this type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Signed<T> {
    pub public_key: PublicKey,
    /// Increments with each `L2Transactions` (Transfer or Withdraw).
    pub nonce: u64,
    pub transaction: T,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary",
           derive(test_strategy::Arbitrary),
           arbitrary(args = (AccountsState, MaxAmount))
           )]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Transfer {
    pub recipient: PublicKey,
    pub amount: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary",
           derive(test_strategy::Arbitrary),
           arbitrary(args = (AccountsState, std::rc::Rc<PublicKey>)))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct L1Deposit {
    #[cfg_attr(feature = "arbitrary",
               strategy(proptest::prelude::any::<proptest::sample::Index>()),
               map(|sampler| args.0.sample_keys(sampler)),
               by_ref
               )]
    pub recipient: PublicKey,

    #[cfg_attr(feature = "arbitrary",
               strategy(proptest::prelude::any::<proptest::sample::Index>()),
               map(|sampler| args.0.deposit(&args.1, #recipient, sampler)),
               )]
    pub amount: u64,
}

/// TODO remove this struct
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Deposit {
    pub amount: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary",
           derive(test_strategy::Arbitrary),
           arbitrary(args = MaxAmount))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Withdraw {
    pub amount: u64,
}

#[cfg(feature = "arbitrary")]
pub use arbitrary_bounds::*;
#[cfg(feature = "arbitrary")]
mod arbitrary_bounds {
    use std::{cell::RefCell, collections::HashMap, fmt, ops::Deref, rc::Rc};

    use super::*;
    use crate::account_trie::Account;

    #[derive(Debug, Default)]
    pub struct PublicKeys(pub Vec<Rc<PublicKey>>);

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
            AccountsState {
                shared: Rc::new(AccountsStateInner {
                    keys: PublicKeys::default(),
                    l1_accounts: RefCell::new(HashMap::new()),
                    l2_accounts: RefCell::new(HashMap::new()),
                }),
            }
        }
    }

    impl AccountsState {
        pub fn sample_keys(&self, sampler: proptest::sample::Index) -> PublicKey {
            let keys = &self.shared.keys;
            keys.0[sampler.index(keys.0.len())].to_vec()
        }

        pub fn deposit(
            &self,
            sender: &PublicKey,
            recipient: &PublicKey,
            amount_sampler: proptest::sample::Index,
        ) -> u64 {
            let mut l1_accounts = self.shared.l1_accounts.borrow_mut();
            let l1_amount = l1_accounts
                .get_mut(sender)
                .expect("sender does not have an l1 account in AccountsState");

            let amount = amount_sampler.index(*l1_amount as usize) as u64;

            let mut l2_accounts = self.shared.l2_accounts.borrow_mut();

            let l2_account = l2_accounts
                .get_mut(recipient)
                .expect("recipient does not have an l2 account in AccountsState");

            // if the deposit will fail don't change the test model state
            if let (Some(l1_bal), Some(l2_bal)) = (
                l1_amount.checked_sub(amount),
                l2_account.balance.checked_add(amount),
            ) {
                *l1_amount = l1_bal;
                l2_account.balance = l2_bal;
            }

            amount
        }
    }

    #[derive(Debug, Default)]
    pub struct MaxAmount(pub u64);
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
