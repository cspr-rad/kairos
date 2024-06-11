#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format, string::String, vec::Vec};

use account_trie::{Account, AccountTrie};
use kairos_trie::{stored::merkle::Snapshot, DigestHasher};
use sha2::Sha256;
use transactions::{KairosTransaction, L1Deposit, Signed, Withdraw};

#[cfg(feature = "rkyv")]
use rkyv::{AlignedVec, Deserialize};

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
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize),
    archive(compare(PartialEq), check_bytes)
)]
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct ProofOutputs {
    pub pre_batch_trie_root: Option<[u8; 32]>,
    pub post_batch_trie_root: Option<[u8; 32]>,
    /// TODO consider replacing with a count and hash of the processed deposits
    pub deposits: Box<[L1Deposit]>,
    pub withdrawals: Box<[Signed<Withdraw>]>,
}

#[cfg(feature = "rkyv")]
impl ProofOutputs {
    pub fn rkyv_serialize(&self) -> AlignedVec {
        rkyv::to_bytes::<_, 256>(self).expect("Failed to rkyv_serialize ProofOutputs")
    }

    pub fn rkyv_deserialize(bytes: &[u8]) -> Result<Self, String> {
        let proof_outputs = rkyv::check_archived_root::<ProofOutputs>(bytes)
            .map_err(|e| format!("Error in rkyv::check_archived_root: {e}"))?;

        let proof_outputs = proof_outputs
            .deserialize(&mut rkyv::Infallible)
            .map_err(|e| format!("Error in ProofOutputs::rkyv_deserialize: {e}"))?;

        Ok(proof_outputs)
    }
}

impl ProofInputs {
    pub fn run_batch_proof_logic(self) -> Result<ProofOutputs, String> {
        let ProofInputs {
            transactions,
            trie_snapshot,
        } = self;

        let hasher = &mut DigestHasher::<Sha256>::default();

        let mut trie = AccountTrie::new_try_from_snapshot(&trie_snapshot)?;
        let pre_batch_trie_root = trie.txn.calc_root_hash(hasher)?.into();

        let (deposits, withdrawals) = trie.apply_batch(
            // TODO Replace with Box<[T]>: IntoIterator once Rust 2024 is stable
            Vec::from(transactions).into_iter(),
        )?;

        let post_batch_trie_root = trie.txn.calc_root_hash(hasher)?.into();

        Ok(ProofOutputs {
            pre_batch_trie_root,
            post_batch_trie_root,
            deposits,
            withdrawals,
        })
    }
}
