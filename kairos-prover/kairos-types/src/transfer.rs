use casper_types::{account::AccountHash, U512};
use serde::{Deserialize, Serialize};
extern crate alloc;
use alloc::vec::Vec;

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
                            // pub merkle_proof: MerkleProof -> tbd.
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
