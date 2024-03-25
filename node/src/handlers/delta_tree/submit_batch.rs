use axum::extract::Json;
use crate::domain::models::transfers;
use crate::domain::models::deposits;
use crate::CONFIG;
use crate::AppState;

use kairos_risc0_types::{KairosDeltaTree, hash_bytes, Transfer, Deposit, Withdrawal, TransactionBatch, RiscZeroProof, CircuitArgs, CircuitJournal};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use serde::{Deserialize, Serialize};
use bincode;
use layer_one_utils::deployments::{post, query};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct SubmitBatch {
// }

#[derive(Serialize)]
pub struct BatchResponse {
    pub status: String
}

// Checks the height of the tree on-chain and if the height is 0 use default tree
// else grab the altest proof and grab output tree
// then use batch and most recent tree to generate, approve and submit to verifier contract
// generate proof
pub async fn submit_batch(State(AppState): State<AppState>) -> impl IntoResponse {
    let state = State(AppState);
    let transfers_filter = transfers::TransfersFilter { processed: Some(false), sender: None, recipient: None };
    let unprocessed_transfers = transfers::get_all(state.pool.clone(), transfers_filter).await.unwrap();
    let transfers: Vec<Transfer> = unprocessed_transfers.into_iter().map(|model| model.into()).collect();

    let deposits_filter = deposits::DepositFilter { processed: Some(false), account: None };
    let unprocessed_deposits = deposits::get_all(state.pool.clone(), deposits_filter).await.unwrap();
    let deposits: Vec<Deposit> = unprocessed_deposits.into_iter().map(|model| model.into()).collect();
    
    // this counter uref is different!
    let tree_index = query::query_counter(&CONFIG.node_address(), &CONFIG.node.port.to_string(), &CONFIG.node.tree_counter_uref).await;
    let mut previous_tree: KairosDeltaTree = KairosDeltaTree{
        zero_node: hash_bytes(vec![0;32]),
        zero_levels: Vec::new(),
        filled: vec![vec![];5],
        root: None,
        index: 0,
        depth: 5
    };
    // get last snapshot if the index > 0!
    if tree_index > 0 {
        let last_proof = query::query_proof(&CONFIG.node_address(), &CONFIG.node.port.to_string(), &CONFIG.node.dict_uref, tree_index.to_string()).await;
        let receipt_deserialized: Receipt = bincode::deserialize(&last_proof.receipt_serialized).unwrap();
        let journal: CircuitJournal = receipt_deserialized.journal.decode::<CircuitJournal>().unwrap();
        previous_tree = journal.output;
    }
    else{
        previous_tree.calculate_zero_levels();
    }

    // construct a batch
    let batch: TransactionBatch = TransactionBatch{
        deposits,
        transfers,
        // empty withdrawal vec
        withdrawals: vec![]
    };

    // now that we have the previous tree, we generate the proof -> needs the prove_batch function that can be found in the verifier contract test_fixture!
    let proof: RiscZeroProof = post::prove_batch(previous_tree, batch);
    // submit the bincode serialized proof to L1
    let payload = bincode::serialize(&proof).unwrap(); // handle error
    post::submit_delta_tree_batch(&CONFIG.node_address(), &CONFIG.node.port.to_string(), &CONFIG.node.secret_key_path, &CONFIG.node.chain_name, &CONFIG.node.verifier_contract, payload).await;

    (StatusCode::OK, "Batch processed successfully").into_response()
}