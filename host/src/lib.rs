use methods::{
    NATIVE_CSPR_TX_ELF, NATIVE_CSPR_TX_ID
};
use serde::{Serialize, Deserialize};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use kairos_risc0_types::{MockLayerTwoStorage, TornadoTree, HashableStruct, TransactionHistory, Transaction, CircuitArgs, CircuitJournal, MockAccounting, ToBytes, Key, U512, hash_bytes};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct RiscZeroProof{
    pub receipt: Receipt,
    pub program_id: Vec<u32>
}

pub fn prove_state_transition(tree: TornadoTree, mock_storage: MockLayerTwoStorage) -> RiscZeroProof{
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
    let proof: RiscZeroProof = prove_state_transition(tree, mock_storage);
    let journal: &CircuitJournal = &proof.receipt.journal.decode::<CircuitJournal>().unwrap();
    println!("Journal: {:?}", &journal);
}
