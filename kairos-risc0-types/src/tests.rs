#[cfg(feature = "tornado-tree")]
#[test]
fn try_hash_batch(){
    use crate::{Deposit, Withdrawal, Transfer, TransactionBatch, HashableStruct};
    let transfers: Vec<Transfer> = vec![];
    let deposits: Vec<Deposit> = vec![];
    let withdrawals: Vec<Withdrawal> = vec![];
    let batch = TransactionBatch{
        transfers,
        deposits,
        withdrawals
    };
    println!("Transaction History Hash: {:?}", batch.hash());
}