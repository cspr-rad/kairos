use casper_types::{bytesrepr::ToBytes, Key, U512};
use serde::{Deserialize, Serialize};
extern crate alloc;
use alloc::{string::String, vec::Vec};
use merkle_tree::{full::MerkleTree, helpers::hash_bytes};
#[derive(Serialize, Deserialize)]
pub struct MerkleProof {
    pub path: Vec<(Vec<u8>, u8)>,
    pub leaf: LayerTwoTransactionBatch
}

#[derive(Serialize, Deserialize)]
pub struct LayerTwoTransactionBatch{
    pub transactions: Vec<LayerTwoTransaction>
}

#[derive(Serialize, Deserialize)]
pub enum LayerTwoTransaction {
    Deposit {
        account: Key,
        amount: U512,
    },
    Withdrawal {
        account: Key,
        amount: U512,
    },
    Transfer {
        sender: Key,
        recipient: Key,
        amount: U512,
        signature: Vec<u8>,
    },
}

impl LayerTwoTransaction {
    pub fn hash(&self) -> Vec<u8> {
        match self {
            LayerTwoTransaction::Deposit {
                account, amount, ..
            } => {
                let mut preimage: Vec<u8> = account.to_bytes().unwrap();
                preimage.append(&mut amount.to_bytes().unwrap());
                hash_bytes(preimage)
            }
            LayerTwoTransaction::Withdrawal {
                account, amount, ..
            } => {
                let mut preimage: Vec<u8> = account.to_bytes().unwrap();
                preimage.append(&mut amount.to_bytes().unwrap());
                hash_bytes(preimage)
            }
            LayerTwoTransaction::Transfer {
                sender,
                recipient,
                amount,
                ..
            } => {
                let mut preimage: Vec<u8> = sender.to_bytes().unwrap();
                preimage.append(&mut recipient.to_bytes().unwrap());
                preimage.append(&mut amount.to_bytes().unwrap());
                hash_bytes(preimage)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LayerTwoAccount {
    pub account: Key,
    pub amount: U512,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerkleRoot {
    pub hash: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MockLayerOneState {
    pub transfers_root: MerkleRoot,
    pub balances_root: MerkleRoot,
}

#[derive(Serialize, Deserialize)]
pub struct MockLayerTwoState {
    pub balances: Vec<LayerTwoAccount>,
}

/*
   * What happens Before Circuit
       * aggregate transactions
       * create transaction_batch from set of transactions and append merkle tree
       * update balances state & append balance tree by H(new_balances)
       -> transaction_batch w. merkle_proof
       -> new_balances leaf w. merkle_proof


   * What happens Inside Circuit
       * Proves that Transfer batch leaf’s merkle path is valid for resulting root
       * Proves that Balances leaf’s merkle path is valid for resulting root


       * Private circuit inputs:
           * New balances leaf with merkle_proof
           * signed transactions, transaction_batch leaf with merkle_proof
       * Circuit Journal:
           * New transfers root
           * New balances root


   * What gets committed to L1
       * Proof
       * Journal
*/
