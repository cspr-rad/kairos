use casper_types::{account::AccountHash, U512};
use serde::{Deserialize, Serialize};
extern crate alloc;
use alloc::{vec::Vec, string::String};

#[derive(Serialize, Deserialize)]
pub struct MerkleProof{
    pub path: Vec<String>,
    pub lr: Vec<bool> // true: H(left+right), false: H(right+left)
}

pub enum LayerTwoTransaction {
    Deposit {
        deploy_status: bool,
        account: AccountHash,
        amount: U512,
    },
    Withdraw {
        deploy_status: bool,
        account: AccountHash,
        amount: U512,
    },
    Transfer {
        sender: AccountHash,
        recipient: AccountHash,
        amount: U512,
        signature: Vec<u8>, // serialized signature -> replace by struct?,
        merkle_proof: Option<MerkleProof>
    },
}

#[derive(Serialize, Deserialize)]
pub struct LayerTwoAccount {
    pub account: AccountHash,
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
    pub accounts: Vec<LayerTwoAccount>,
}

/*
    * What happens Before Circuit
        * aggregate transactions
        * create transaction_batch from set of transactions and append merkle tree
        * update balances state & append balance tree by H(new_balances)
        -> transaction_batch w. merkle_proof
        -> new_balances leaf w. merkle_proof

    * What happens Inside Circuit
        * Proves that Transfer merkle paths are valid for resulting root hash
        * Proves that Balances are valid and result from a set of transfers

        * Private circuit inputs:
            * New balances leaf with merkle_proof
            * Set of transaction_batch leaf with merkle_proof
        * Circuit Journal:
            * New transfers root
            * New balances root

    * What gets committed to L1
        * Proof
        * Journal
*/