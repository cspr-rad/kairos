use serde::{Serialize, Deserialize};
use risc0_zkvm::Receipt;
pub use tornado_tree_rs::{TornadoTree, crypto::hash_bytes};
use casper_types::{bytesrepr::ToBytes, Key, U512};
use serde_json;
use std::collections::HashMap;
mod tests;
// The decision to use "Key" over  more deterministic types is based on the variable design
// with respect to target node architecture. Everything will be handled in Bytes and Keys.

pub trait HashableStruct{
    fn hash(&self) -> Vec<u8>;
}

#[derive(Serialize, Deserialize)]
pub struct RiscZeroProof{
    pub receipt: Receipt,
    pub program_id: Vec<u32>
}

#[derive(Serialize, Deserialize)]
struct MockLayerTwoStorage {
    balances: MockAccounting,
    transactions: TransactionHistory
}

impl HashableStruct for MockLayerTwoStorage{
    fn hash(&self) -> Vec<u8>{
        hash_bytes(serde_json::to_vec(self).unwrap())
    }
}

#[derive(Serialize, Deserialize)]
pub struct MockAccounting{
    balances: HashMap<Key, U512>,
}
impl HashableStruct for MockAccounting{
    fn hash(&self) -> Vec<u8>{
        hash_bytes(serde_json::to_vec(self).unwrap())
    }
}

#[derive(Serialize, Deserialize)]
pub struct TransactionHistory{
    pub transactions: Vec<Transaction>
}

// This will likely not be used, since hashing the Balance state will be sufficient
// New leaf in the tree <- New Balance state
impl HashableStruct for TransactionHistory{
    fn hash(&self) -> Vec<u8>{
        hash_bytes(serde_json::to_vec(self).unwrap())
    }
}


#[derive(Serialize, Deserialize)]
pub enum Transaction {
    Deposit {
        account: Key,
        amount: U512,
        processed: bool,
        id: u64
    },
    Withdrawal {
        account: Key,
        amount: U512,
        processed: bool,
        id: u64
    },
    Transfer {
        sender: Key,
        recipient: Key,
        amount: U512,
        signature: Vec<u8>,
        processed: bool,
        nonce: u64
    },
}
impl HashableStruct for Transaction{
    fn hash(&self) -> Vec<u8> {
        match self {
            Transaction::Deposit {
                account, amount, processed, id
            } | Transaction::Withdrawal{
                account, amount, processed, id
            } => {
                let mut preimage: Vec<u8> = account.to_bytes().unwrap();
                preimage.append(&mut amount.to_bytes().unwrap());
                preimage.append(&mut id.to_bytes().unwrap());
                hash_bytes(preimage)
            }
            Transaction::Transfer {
                sender,
                recipient,
                amount,
                signature,
                processed,
                nonce
            } => {
                let mut preimage: Vec<u8> = sender.to_bytes().unwrap();
                preimage.append(&mut recipient.to_bytes().unwrap());
                preimage.append(&mut amount.to_bytes().unwrap());
                preimage.append(&mut nonce.to_bytes().unwrap());
                hash_bytes(preimage)
            }
        }
    }
}


/* The storage component is simplified on purpose for quick prototyping!

The circuit will execute a bunch of tasks in order:

    1. Deposits
        - increase the account balance
    2. Transfers
        - verify the transaction signature
        - apply the change in balance for the affected accounts
    3. Withdrawals
        todo!
    
    Hash the Balance State and insert it as a new leaf to the TornadoTree
    ! For the client to be able to verify the inclusion of the leaf a merkle proof must be created.
    ! Public input must be the old Tree

    => The contract must compare the public input Tree to its current Tree and revert if there is a mismatch.
*/

#[derive(Serialize, Deserialize)]
pub struct CircuitInput{
    pub tornado: TornadoTree,
    pub mock_storage: MockLayerTwoStorage,
}