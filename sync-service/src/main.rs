use methods::{
    NATIVE_CSPR_TX_ELF, NATIVE_CSPR_TX_ID
};
use serde::{Serialize, Deserialize};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use kairos_risc0_types::{hash_bytes, constants::{FORMATTED_COUNTER_UREF, FORMATTED_DEFAULT_ACCOUNT_STR, FORMATTED_DICT_UREF, NODE_ADDRESS, RPC_PORT}, CircuitArgs, CircuitJournal, HashableStruct, Key, ToBytes, KairosDeltaTree, Transfer, Deposit, Withdrawal, U512, URef};
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
    loop{
        // sync deposits
        sleep(Duration::from_millis(10000));
        // add some sort of timeout here
    }
}

#[tokio::main]
async fn main(){
    // start service
}