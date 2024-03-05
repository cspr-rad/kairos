use crate::constants::{FORMATTED_DEFAULT_ACCOUNT_STR, PATH_TO_MOCK_STATE_FILE, PATH_TO_MOCK_TREE_FILE};

#[cfg(feature = "tornado-tree")]
#[test]
fn try_hash_batch(){
    use crate::{Deposit, Withdrawal, Transfer, TransactionBatch, HashableStruct};
    use casper_types::{bytesrepr::ToBytes, Key, U512};
    use std::collections::HashMap;
    let transfers: Vec<Transfer> = vec![];
    let deposits: Vec<Deposit> = vec![];
    let withdrawals: Vec<Withdrawal> = vec![];
    let mut batch = TransactionBatch{
        transfers,
        deposits,
        withdrawals
    };
    println!("Transaction History Hash: {:?}", batch.hash());
}