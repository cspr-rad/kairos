#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{boxed::Box, string::String, vec::Vec};

use account_trie::{Account, AccountTrie};
use kairos_trie::{stored::merkle::Snapshot, DigestHasher, NodeHash, TrieRoot};
use sha2::Sha256;
use transactions::{KairosTransaction, L1Deposit, Signed, Withdraw};

pub mod account_trie;
pub mod transactions;

/// `ProofInputs` contains the minimum logical inputs needed to apply a batch of transactions.
///
/// The trie snapshot holds the pre-batch merkle root.
/// The post-batch merkle root is given running `self.run_batch_proof_logic`
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProofInputs {
    pub transactions: Box<[KairosTransaction]>,

    pub trie_snapshot: Snapshot<Account>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct ProofOutputs {
    pub pre_batch_trie_root: TrieRoot<NodeHash>,
    pub post_batch_trie_root: TrieRoot<NodeHash>,
    /// TODO consider replacing with a count and hash of the processed deposits
    pub deposits: Box<[L1Deposit]>,
    pub withdrawals: Box<[Signed<Withdraw>]>,
}

impl ProofInputs {
    pub fn run_batch_proof_logic(self) -> Result<ProofOutputs, String> {
        let ProofInputs {
            transactions,
            trie_snapshot,
        } = self;

        let hasher = &mut DigestHasher::<Sha256>::default();

        let mut trie = AccountTrie::new_try_from_snapshot(&trie_snapshot)?;
        let pre_batch_trie_root = trie.txn.calc_root_hash(hasher)?;

        let (deposits, withdrawals) = trie.apply_batch(
            // Replace with Box<[T]>: IntoIterator once Rust 2024 is stable
            Vec::from(transactions).into_iter(),
        )?;

        let post_batch_trie_root = trie.txn.calc_root_hash(hasher)?;

        Ok(ProofOutputs {
            pre_batch_trie_root,
            post_batch_trie_root,
            deposits,
            withdrawals,
        })
    }
}
