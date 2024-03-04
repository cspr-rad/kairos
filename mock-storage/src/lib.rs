use kairos_risc0_types::{MockLayerTwoStorage, TornadoTree, HashableStruct, TransactionHistory, Transaction, CircuitArgs, CircuitJournal, MockAccounting, ToBytes, Key, U512, hash_bytes};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json;

use std::fs::OpenOptions;
use std::io::{self, Write};

use anyhow::Result;

pub fn init_storage() -> Result<()>{
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("/Users/chef/Desktop/kairos-risc0/mock-store.dat")?;
    Ok(())
}


fn write_struct_to_file(mock_store: MockLayerTwoStorage) -> std::io::Result<()> {
    let serialized = serde_json::to_string(&mock_store).unwrap();
    let mut file = File::create("/Users/chef/Desktop/kairos-risc0/mock-store.dat")?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

fn read_struct_from_file() -> std::io::Result<MyStruct> {
    let mut file = File::open("/Users/chef/Desktop/kairos-risc0/mock-store.dat")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let my_struct: MyStruct = serde_json::from_str(&contents).unwrap();
    Ok(my_struct)
}

pub fn setup_network() -> (TornadoTree, MockLayerTwoStorage){
    let mut tree: TornadoTree = TornadoTree{
        zero_node: hash_bytes(vec![0;32]),
        zero_levels: Vec::new(),
        filled: vec![vec![], vec![], vec![], vec![], vec![]],
        index: 0,
        depth: 5
    };
    tree.calculate_zero_levels();
    let mock_storage: MockLayerTwoStorage = MockLayerTwoStorage{
        balances: MockAccounting{
            balances: HashMap::new()
        },
        transactions: TransactionHistory{
            transactions: vec![
                Transaction::Deposit{
                    account: Key::from_formatted_str("account-hash-32da6919b3a0a9be4bc5b38fa74de98f90dc43924bf17e73f6635992f110f011").unwrap(),
                    amount: U512::from(1u64),
                    processed: false,
                    id: 0
                },
            ]
        },
    };
    (tree, mock_storage)
}

// todo: implement storage 
// for the first iteration of the demo it is sufficient to just write and read the MockLayerTwoStorage and TornadoTree
// later the tree will be constructed inside the L1 contract from which it can be queried.
// finally the transactions are stored in an actual DB and only pending Transactions and affected Balances are submitted to the circuit / batched.
// for each batch the pending transactions and affected Balances must fit in memory.

