use crate::constants::{FORMATTED_DEFAULT_ACCOUNT_STR, PATH_TO_MOCK_STATE_FILE, PATH_TO_MOCK_TREE_FILE};

#[test]
fn try_hash_batch(){
    use crate::{Transaction, TransactionHistory, HashableStruct};
    use casper_types::{bytesrepr::ToBytes, Key, U512};
    use std::collections::HashMap;
    let mut transactions: HashMap<String, Transaction> = HashMap::new();
    transactions.insert("0".to_string(), Transaction::Deposit{
        account: Key::from_formatted_str(FORMATTED_DEFAULT_ACCOUNT_STR).unwrap(),
        amount: U512::from(0u64),
        processed: false,
        id: 0
    });
    transactions.insert("1".to_string(), Transaction::Transfer{
        sender: Key::from_formatted_str(FORMATTED_DEFAULT_ACCOUNT_STR).unwrap(),
        recipient: Key::from_formatted_str(FORMATTED_DEFAULT_ACCOUNT_STR).unwrap(),
        amount: U512::from(0),
        signature: vec![],
        processed: false,
        nonce: 0
    });
    let mut batch = TransactionHistory{
        transactions
    };
    println!("Transaction History Hash: {:?}", batch.hash());
}

#[test]
fn try_initialize_mock_state(){
    use crate::{Transaction, TransactionHistory, HashableStruct};
    use casper_types::{bytesrepr::ToBytes, Key, U512};
    use crate::{TornadoTree, hash_bytes};
}

#[test]
fn try_insert_state_as_leaf(){
    // insert the Mock state into the Tornado Tree
}