#![no_main]
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);
use kairos_risc0_types::{CircuitArgs, CircuitJournal, HashableStruct, KairosDeltaTree, TransactionBatch};

pub fn main() {
    // todo: include the input tree in the circuit journal
    let inputs: CircuitArgs = env::read();
    let batch: TransactionBatch = inputs.batch;
    let input_tree_clone: KairosDeltaTree = inputs.tree.clone();
    let mut tree: KairosDeltaTree = inputs.tree;

    for transfer in &batch.transfers {
        //todo: check the transfer signature
    }

    // hash the batch and add it to the tornado tree
    let new_leaf: Vec<u8> = batch.hash();
    tree.add_leaf(new_leaf);
    let journal: CircuitJournal = CircuitJournal{
        input: input_tree_clone,
        output: Some(tree)
    };
    env::commit(&journal);
}
