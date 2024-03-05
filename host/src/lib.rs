use methods::{
    NATIVE_CSPR_TX_ELF, NATIVE_CSPR_TX_ID
};
use serde::{Serialize, Deserialize};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use kairos_risc0_types::{MockLayerTwoStorage, TornadoTree, HashableStruct, TransactionHistory, Transaction, CircuitArgs, CircuitJournal, MockAccounting, ToBytes, Key, U512, hash_bytes, constants::{FORMATTED_DEFAULT_ACCOUNT_STR, PATH_TO_MOCK_STATE_FILE, PATH_TO_MOCK_TREE_FILE}};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct RiscZeroProof{
    pub receipt: Receipt,
    pub program_id: Vec<u32>
}

pub fn prove_state_transition(tree: TornadoTree, mock_storage: MockLayerTwoStorage) -> RiscZeroProof{
    /*
        todo: sort transactions in order Deposits->Transers(->Withdrawals)
    */
    env_logger::init();
    let inputs = CircuitArgs{
        tornado: tree,
        mock_storage
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

    let mut transactions: HashMap<String, Transaction> = HashMap::new();
    transactions.insert("0".to_string(), Transaction::Deposit{
        account: Key::from_formatted_str(FORMATTED_DEFAULT_ACCOUNT_STR).unwrap(),
        amount: U512::from(0u64),
        processed: false,
        id: 0
    });
    transactions.insert("1".to_string(), Transaction::Transfer{
        sender: Key::from_formatted_str(FORMATTED_DEFAULT_ACCOUNT_STR).unwrap(),
        recipient: Key::from_formatted_str(FORMATTED_DEFAULT_ACCOUNT_STR).unwrap(),
        amount: U512::from(0),
        signature: vec![],
        processed: false,
        nonce: 0
    });
    let mut batch = TransactionHistory{
        transactions
    };
    
    let mock_storage: MockLayerTwoStorage = MockLayerTwoStorage{
        balances: MockAccounting{
            balances: HashMap::new()
        },
        transactions: batch
    };
    let proof: RiscZeroProof = prove_state_transition(tree, mock_storage);
    let journal: &CircuitJournal = &proof.receipt.journal.decode::<CircuitJournal>().unwrap();
    println!("Journal: {:?}", &journal);
}
