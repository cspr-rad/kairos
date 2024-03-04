use methods::{
    NATIVE_CSPR_TX_ELF, NATIVE_CSPR_TX_ID
};
use serde::{Serialize, Deserialize};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use kairos_risc0_types::{MockLayerTwoStorage, TornadoTree, HashableStruct, TransactionHistory, Transaction, CircuitArgs, CircuitJournal, MockAccounting, ToBytes, Key, U512, hash_bytes};
use std::collections::HashMap;

fn setup_network() -> (TornadoTree, MockLayerTwoStorage){
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

/* Current development goal:
    1. Monitor Deposits on L1 and add them to the MockLayerTwoStorage
    2. Only process transactions with processed=False
    3. Flag those transactions that have been included in a batch with processed=True

    4. Accept Transfers via CLI / Rest (without signatures) - reject if L2 Balance insufficient
    5. Generate proofs for Deposits & Transfers and mutate the L2 state
    6. Implement Transfer signatures
*/

fn main(){
    let state: (TornadoTree, MockLayerTwoStorage) = setup_network();
    /* Storage
        implement a simple storage for 'state' - mysql or even just a file-based I/O script
        add new transactions, update the balances and set the 'processed' flag in storage
    */


    // this counter is supposed to live on the blockchain, 
    // for mock development it can be stored in memory.
    let deposit_index: u64 = 0;
    // todo: start process that monitors the L1 for Deposits

    // todo: accept Transfers
    // todo: batch Transactions
    // todo: generate proofs for Batches
}