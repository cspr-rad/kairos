use methods::{
    NATIVE_CSPR_TX_ELF, NATIVE_CSPR_TX_ID
};
use serde::{Serialize, Deserialize};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use kairos_risc0_types::{constants::{FORMATTED_DEFAULT_ACCOUNT_STR, PATH_TO_MOCK_STATE_FILE, PATH_TO_MOCK_TREE_FILE}, hash_bytes, CircuitArgs, CircuitJournal, Deposit, HashableStruct, Key, ToBytes, TornadoTree, TransactionBatch, Transfer, Withdrawal, U512};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct RiscZeroProof{
    pub receipt: Receipt,
    pub program_id: Vec<u32>
}

pub fn prove_state_transition(tree: TornadoTree, batch: TransactionBatch) -> RiscZeroProof{
    /*
        todo: sort transactions in order Deposits->Transers(->Withdrawals)
    */
    env_logger::init();
    let inputs = CircuitArgs{
        tornado: tree,
        batch: batch
    };
    let env = ExecutorEnv::builder()
    .write(&inputs)
    .unwrap()
    .build()
    .unwrap();

    let prover = default_prover();
    let receipt = prover.prove(env, NATIVE_CSPR_TX_ELF).unwrap();
    receipt.verify(NATIVE_CSPR_TX_ID).expect("Failed to verify proof!");
    RiscZeroProof{
        receipt,
        program_id: NATIVE_CSPR_TX_ID.to_vec()
    }
}

#[test]
fn test_proof_generation(){
    let mut tree: TornadoTree = TornadoTree{
        zero_node: hash_bytes(vec![0;32]),
        zero_levels: Vec::new(),
        filled: vec![vec![], vec![], vec![], vec![], vec![]],
        root: None,
        index: 0,
        depth: 5
    };
    tree.calculate_zero_levels();
    let transfers: Vec<Transfer> = vec![];
    let deposits: Vec<Deposit> = vec![];
    let withdrawals: Vec<Withdrawal> = vec![];
    let batch: TransactionBatch = TransactionBatch{
        transfers,
        deposits, 
        withdrawals
    };
    
    let proof: RiscZeroProof = prove_state_transition(tree, batch);
    let journal: &TornadoTree = &proof.receipt.journal.decode::<TornadoTree>().unwrap();
    println!("Journal: {:?}", &journal);
}
