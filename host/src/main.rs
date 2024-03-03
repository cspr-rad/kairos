use methods::{
    NATIVE_CSPR_TX_ELF, NATIVE_CSPR_TX_ID
};
use risc0_zkvm::{default_prover, ExecutorEnv};
use tornado_tree_rs::{TornadoTree, crypto::hash_bytes};

/*
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use serde_json;
mod types;
use types::RiscZeroProof;

#[doc(hidden)]
pub fn verify<T: AsRef<[u8]>>(
    proof_serialized: T
) -> [u8;1]{
    let risc0_proof: RiscZeroProof = serde_json::from_slice(&proof_serialized.as_ref()).unwrap();
    let program_id: [u32;8] = risc0_proof.program_id.try_into().unwrap();
    match risc0_proof.receipt.verify(program_id){
        Ok(_) => [1],
        Err(_) => [0]
    }
}
*/

fn main() {
    env_logger::init();
    // initialize an empty tree for testing
    let mut tree: TornadoTree = TornadoTree{
        zero_node: hash_bytes(vec![0;32]),
        zero_levels: Vec::new(),
        filled: vec![vec![], vec![], vec![], vec![], vec![]],
        index: 0,
        depth: 5
    };
    tree.calculate_zero_levels();
    println!("Tree: {:?}", &tree);
    let env = ExecutorEnv::builder()
        .write(&tree)
        .unwrap()
        .build()
        .unwrap();
    let prover = default_prover();
    let receipt = prover.prove(env, NATIVE_CSPR_TX_ELF).unwrap();
    let _output: &Vec<u8> = &receipt.journal.decode::<Vec<u8>>().unwrap();
    println!("Tree root: {:?}", &_output);
    receipt.verify(NATIVE_CSPR_TX_ID).unwrap();
}