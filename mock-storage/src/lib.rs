use kairos_risc0_types::{MockLayerTwoStorage, TornadoTree, HashableStruct, TransactionHistory, Transaction, CircuitArgs, CircuitJournal, MockAccounting, ToBytes, Key, U512, hash_bytes};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json;

use std::fs::{OpenOptions, File};
use std::io::{self, Read, Write};

use anyhow::Result;

pub fn init_mock_state() -> (TornadoTree, MockLayerTwoStorage){
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
            transactions: HashMap::new()
        },
    };
    (tree, mock_storage)
}

trait MutableState{
    fn insert_transaction(&mut self, key: String, transaction: Transaction){}
}

impl MutableState for MockLayerTwoStorage{
    fn insert_transaction(&mut self, key: String, transaction: Transaction){
        self.transactions.transactions.insert(key, transaction);
    }
}

pub struct MockStorage{
    pub path: String
}
impl MockStorage{
    pub fn init_storage(&self) -> Result<()>{
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.path)?;
        Ok(())
    }
    
    pub fn write_struct_to_file<T: Serialize>(&self, mock_store: &T) -> std::io::Result<()> {
        let serialized = serde_json::to_string(&mock_store).unwrap();
        let mut file = File::create(&self.path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }
    
    
    pub fn read_serialized_struct_from_file(&self) -> Result<String>{
        let mut file = File::open(&self.path)?;
        let mut contents = String::new();
        let content = file.read_to_string(&mut contents)?;
        Ok(contents)
    }
}

// todo: implement storage 
// for the first iteration of the demo it is sufficient to just write and read the MockLayerTwoStorage and TornadoTree
// later the tree will be constructed inside the L1 contract from which it can be queried.
// finally the transactions are stored in an actual DB and only pending Transactions and affected Balances are submitted to the circuit / batched.
// for each batch the pending transactions and affected Balances must fit in memory.

#[test]
fn test_init_storage(){
    let mock_state: (TornadoTree, MockLayerTwoStorage) = init_mock_state();
    let tornado_storage = MockStorage{
        path: "/Users/chef/Desktop/kairos-risc0/host/zk-tornado.dat".to_string()
    };
    tornado_storage.init_storage();
    let mock_layer_two_storage = MockStorage{
        path: "/Users/chef/Desktop/kairos-risc0/host/zk-mock.dat".to_string()
    };
    mock_layer_two_storage.init_storage();
}

#[test]
fn test_mock_storage(){
    let mock_state: (TornadoTree, MockLayerTwoStorage) = init_mock_state();
    let tornado_storage = MockStorage{
        path: "/Users/chef/Desktop/kairos-risc0/host/zk-tornado.dat".to_string()
    };
    tornado_storage.init_storage();
    let mock_layer_two_storage = MockStorage{
        path: "/Users/chef/Desktop/kairos-risc0/host/zk-mock.dat".to_string()
    };
    mock_layer_two_storage.init_storage();

    tornado_storage.write_struct_to_file(&mock_state.0);
    mock_layer_two_storage.write_struct_to_file(&mock_state.1);
    let local_tornado = tornado_storage.read_serialized_struct_from_file().unwrap();
    assert_eq!(&serde_json::from_str::<TornadoTree>(&local_tornado).unwrap(), &mock_state.0);
    let local_mock_storage = mock_layer_two_storage.read_serialized_struct_from_file().unwrap();
    assert_eq!(&serde_json::from_str::<MockLayerTwoStorage>(&local_mock_storage).unwrap(), &mock_state.1);
}

#[test]
fn test_insert_transaction(){
    let mock_state: (TornadoTree, MockLayerTwoStorage) = init_mock_state();
    let mut mock_storage = mock_state.1;
    mock_storage.insert_transaction("0".to_string(), Transaction::Deposit { 
        account: Key::from_formatted_str("account-hash-32da6919b3a0a9be4bc5b38fa74de98f90dc43924bf17e73f6635992f110f011").unwrap(), 
        amount: U512::from(0), 
        processed: false, 
        id: 0 
    });
    println!("Mock storage: {:?}", &mock_storage);
}