use methods::{
    NATIVE_CSPR_TX_ELF, NATIVE_CSPR_TX_ID
};
use serde::{Serialize, Deserialize};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use kairos_risc0_types::{MockLayerTwoStorage, TornadoTree, HashableStruct, TransactionHistory, Transaction, CircuitArgs, CircuitJournal, MockAccounting, ToBytes, Key, U512, hash_bytes};
use kairos_contract_cli::deployments::{get_deposit_event, get_counter};
use std::collections::HashMap;

// This Rest(Rocket) API will have 2 entry points for the first iteration of the demo:
/*

    Transfer -> inserts a transfer into the local mock storage

    Submit -> produces a proof for current batch and submits it to the L1

*/

pub fn main(){

}