use methods::{
    NATIVE_CSPR_TX_ELF, NATIVE_CSPR_TX_ID
};
use serde::{Serialize, Deserialize};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use kairos_risc0_types::{MockLayerTwoStorage, TornadoTree, HashableStruct, TransactionHistory, Transaction, CircuitArgs, CircuitJournal, MockAccounting, ToBytes, Key, U512, hash_bytes, constants::{FORMATTED_DEFAULT_ACCOUNT_STR, PATH_TO_MOCK_STATE_FILE, PATH_TO_MOCK_TREE_FILE}};
use kairos_contract_cli::deployments::{get_deposit_event, get_counter};
use std::collections::HashMap;

// This Rest(Rocket) API will have 2 entry points for the first iteration of the demo:
/*

    Transfer -> inserts a transfer into the local mock storage

    Submit -> produces a proof for current batch and submits it to the L1
    How is a proof produced?
        1. the local storage is read (current tree, current mock state which includes balances and transactions)
        2. the prove_state_transition function in 'host' is called with the local state as input
        3. the proof struct is returned that can be submitted to the L1 using the L1 client.
        The contract being called is the Tree/State contract, not the deposit contract!
        The Tree/State contract utilizes the on-chain verifier / host function
*/

pub fn main(){

}