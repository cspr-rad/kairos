use methods::{
    NATIVE_CSPR_TX_ELF, NATIVE_CSPR_TX_ID
};
use serde::{Serialize, Deserialize};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use kairos_risc0_types::{hash_bytes, constants::{FORMATTED_COUNTER_UREF, FORMATTED_DEFAULT_ACCOUNT_STR, FORMATTED_DICT_UREF, NODE_ADDRESS, PATH_TO_MOCK_STATE_FILE, PATH_TO_MOCK_TREE_FILE, RPC_PORT}, CircuitArgs, CircuitJournal, HashableStruct, Key, ToBytes, TornadoTree, Transfer, Deposit, Withdrawal, U512};
use casper_types::URef;
use kairos_contract_cli::deployments::{get_deposit_event, get_counter};
// should be same as deposit.
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

async fn await_deposits(){
    // store L2 index in memory for testing
    let deposit_index: u128 = 0;
    loop{
        let on_chain_height = get_counter::get(NODE_ADDRESS, RPC_PORT.to_string(), URef::from_formatted_str(FORMATTED_COUNTER_UREF).unwrap()).await;
        // fetch all new L1 deposits
        if on_chain_height > deposit_index.into(){
            for i in deposit_index..on_chain_height.as_u128(){
                // get the deposit and insert it into local storage / apply the L2 balance changes
                // store deposit locally
                // todo: add Key / identifier / height to Deposit struct
                // storage and state identifiers are quite confusing, should be more concise in the future.
            }
        }
        // check every 10 seconds for simple demo
        sleep(Duration::from_millis(10000));
        // add some sort of timeout here
    }
}

#[tokio::main]
async fn main(){
    // start service
}