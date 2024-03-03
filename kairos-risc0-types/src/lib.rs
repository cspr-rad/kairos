use serde::{Serialize, Deserialize};
use risc0_zkvm::Receipt;
pub use tornado_tree_rs::{TornadoTree, crypto::hash_bytes};
use casper_types::{bytesrepr::ToBytes, Key, U512};
use serde_json;
use std::collections::HashMap;

// The decision to use "Key" over  more deterministic types is based on the variable design
// with respect to target node architecture. Everything will be handled in Bytes and Keys.

#[derive(Serialize, Deserialize)]
pub struct RiscZeroProof{
    pub receipt: Receipt,
    pub program_id: Vec<u32>
}

// temporary solution, ideally these are not submitted to the L1
#[derive(Serialize, Deserialize)]
struct Accounting {
    balances: HashMap<Key, U512>,
}

#[derive(Serialize, Deserialize)]
pub enum Transaction {
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
impl Transaction {
    pub fn hash(&self) -> Vec<u8> {
        match self {
            Transaction::Deposit {
                account, amount, ..
            } | Transaction::Withdrawal{
                account, amount, ..
            }=> {
                let mut preimage: Vec<u8> = account.to_bytes().unwrap();
                preimage.append(&mut amount.to_bytes().unwrap());
                hash_bytes(preimage)
            }
            Transaction::Transfer {
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
pub struct TransactionBatch{
    pub transactions: Vec<Transaction>
    // todo: add Withdrawal
}
impl TransactionBatch{
    pub fn hash(&mut self) -> Vec<u8>{
        hash_bytes(serde_json::to_vec(self).unwrap())
    }
}


#[derive(Serialize, Deserialize)]
pub struct CircuitInput{
    pub tornado: TornadoTree,
    pub leaf: Vec<u8>
}

#[test]
fn try_hash_batch(){
    let transactions = vec![
        Transaction::Deposit{
            account: Key::from_formatted_str("account-hash-32da6919b3a0a9be4bc5b38fa74de98f90dc43924bf17e73f6635992f110f011").unwrap(),
            amount: U512::from(0u64)
        },
        Transaction::Transfer{
            sender: Key::from_formatted_str("account-hash-32da6919b3a0a9be4bc5b38fa74de98f90dc43924bf17e73f6635992f110f011").unwrap(),
            recipient: Key::from_formatted_str("account-hash-32da6919b3a0a9be4bc5b38fa74de98f90dc43924bf17e73f6635992f110f011").unwrap(),
            amount: U512::from(0),
            signature: vec![]
        }
    ];
    let mut batch = TransactionBatch{
        transactions
    };
    println!("Batch Hash: {:?}", batch.hash());
}