use methods::{
    NATIVE_CSPR_TX_ELF, NATIVE_CSPR_TX_ID
};
use serde::{Serialize, Deserialize};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use kairos_risc0_types::{MockLayerTwoStorage, TornadoTree, HashableStruct, TransactionHistory, Transaction, CircuitArgs, CircuitJournal, MockAccounting, ToBytes, Key, U512, hash_bytes, onstants::{FORMATTED_DEFAULT_ACCOUNT_STR, PATH_TO_MOCK_STATE_FILE, PATH_TO_MOCK_TREE_FILE}};
use kairos_contract_cli::deployments::{get_deposit_event, get_counter};
use contract_types::Deposit;
use mock_storage::{MockStorage, MutableState};
use std::collections::HashMap;
use std::thread::sleep;
use core::time::Duration;

/* Current development goal:
    1. Monitor Deposits on L1 and add them to the MockLayerTwoStorage
    2. Only process transactions with processed=False
    3. Flag those transactions that have been included in a batch with processed=True

    4. Accept Transfers via CLI / Rest (without signatures) - reject if L2 Balance insufficient
    5. Generate proofs for Deposits & Transfers and mutate the L2 state
    6. Submit proofs to the L1

    IDEA: For Demo purpose, trigger the batch submission manually, through a REST endpoint.

    7. Implement Transfer signatures (not difficult, but pushed back due to it being a straight-forward process)
*/

async fn await_deposits(mock_storage: MockLayerTwoStorage){
    // store L2 index in memory for testing
    let deposit_index: u128 = 0;
    loop{
        let on_chain_height = get_counter::get(node_address, rpc_port, counter_uref).await;
        // fetch all new L1 deposits
        if on_chain_height > deposit_index.into(){
            for i in deposit_index..on_chain_height.as_u128(){
                // get the deposit and insert it into local storage / apply the L2 balance changes
                let deposit: Deposit = get_deposit_event::get(node_address, rpc_port, dict_uref, "0".to_string()).await;
                // store deposit locally
                // todo: add Key / identifier / height to Deposit struct
                // storage and state identifiers are quite confusing, should be more concise in the future.
                mock_storage.insert_transaction("0".to_string(), Transaction::Deposit { 
                    account: deposit.account, 
                    amount: deposit.amount, 
                    processed: false, 
                    id: 0 
                });
            }
        }
        // check every 10 seconds for simple demo
        sleep(Duration::from_millis(10000));
        // add some sort of timeout here
    }
}

/*
    For testing run the service manually alongside the api service
    Either implement a simple CLI or hardcode the node_address, rpc_port, 
    dict_uref according to the local nctl network
*/

#[tokio::main]
async fn main(){
    let mock_storage = MockStorage{
        path: PATH_TO_MOCK_STATE_FILE.to_string()
    };
    await_deposits(mock_storage);
    /* Storage
        implement a simple storage for 'state' - mysql or even just a file-based I/O script
        add new transactions, update the balances and set the 'processed' flag in storage
    */
    // todo: start process that monitors the L1 for Deposits

    // todo: accept Transfers
    // todo: batch Transactions
    // todo: generate proofs for Batches
}