#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{boxed::Box, string::String, vec::Vec};

use account_trie::{Account, AccountTrie};
use kairos_trie::{stored::merkle::Snapshot, DigestHasher, NodeHash, TrieRoot};
use sha2::Sha256;
use transactions::{L1Deposit, L2Transactions, Signed, Withdraw};

pub mod account_trie;
pub mod transactions;

/// `ProofInputs` contains the minimum logical inputs needed to apply a batch of transactions.
///
/// The trie snapshot holds the pre-batch merkle root.
/// The post-batch merkle root is given running `self.run_batch_proof_logic`
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProofInputs {
    pub l1_deposits: Box<[L1Deposit]>,
    pub l2_transactions: Box<[Signed<L2Transactions>]>,

    pub trie_snapshot: Snapshot<Account>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct ProofOutputs {
    pub trie_root: TrieRoot<NodeHash>,
    pub withdrawals: Box<[Signed<Withdraw>]>,
}

impl ProofInputs {
    pub fn new(
        l1_deposits: impl Into<Box<[L1Deposit]>>,
        l2_transactions: impl Into<Box<[Signed<L2Transactions>]>>,
        trie_snapshot: Snapshot<Account>,
    ) -> Self {
        Self {
            l1_deposits: l1_deposits.into(),
            l2_transactions: l2_transactions.into(),
            trie_snapshot,
        }
    }

    pub fn run_batch_proof_logic(self) -> Result<ProofOutputs, String> {
        let ProofInputs {
            l1_deposits,
            l2_transactions,
            trie_snapshot,
        } = self;

        let mut trie = AccountTrie::new_try_from_snapshot(&trie_snapshot)?;

        let withdrawals = trie.apply_batch(
            // Replace with Box<[T]>: IntoIterator once Rust 2024 is stable
            Vec::from(l1_deposits).into_iter(),
            Vec::from(l2_transactions).into_iter(),
        )?;

        let trie_root = trie
            .txn
            .calc_root_hash(&mut DigestHasher::<Sha256>::default())?;

        Ok(ProofOutputs {
            trie_root,
            withdrawals,
        })
    }
}
