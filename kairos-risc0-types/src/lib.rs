use serde::{Serialize, Deserialize};
use risc0_zkvm::Receipt;
pub use tornado_tree_rs::TornadoTree;

#[derive(Serialize, Deserialize)]
pub struct RiscZeroProof{
    pub receipt: Receipt,
    pub program_id: Vec<u32>
}

/*
#[derive(Serialize, Deserialize)]
pub struct TransactionBatch{

}
*/

#[derive(Serialize, Deserialize)]
pub struct CircuitInput{
    tornado: TornadoTree,
    leaf: Vec<u8>
}