#[test]
fn try_hash_batch(){
    use crate::{Transaction, TransactionHistory, HashableStruct};
    use casper_types::{bytesrepr::ToBytes, Key, U512};
    let transactions = vec![
        Transaction::Deposit{
            account: Key::from_formatted_str("account-hash-32da6919b3a0a9be4bc5b38fa74de98f90dc43924bf17e73f6635992f110f011").unwrap(),
            amount: U512::from(0u64),
            processed: false,
            id: 0
        },
        Transaction::Transfer{
            sender: Key::from_formatted_str("account-hash-32da6919b3a0a9be4bc5b38fa74de98f90dc43924bf17e73f6635992f110f011").unwrap(),
            recipient: Key::from_formatted_str("account-hash-32da6919b3a0a9be4bc5b38fa74de98f90dc43924bf17e73f6635992f110f011").unwrap(),
            amount: U512::from(0),
            signature: vec![],
            processed: false,
            nonce: 0
        }
    ];
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