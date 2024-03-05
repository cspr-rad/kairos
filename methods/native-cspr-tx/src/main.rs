#![no_main]
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);
use kairos_risc0_types::{CircuitArgs, CircuitJournal, HashableStruct, TornadoTree, TransactionBatch, U512};

pub fn main() {
    let mut inputs: CircuitArgs = env::read();
    let mut circuit_journal = CircuitJournal{
        input: inputs.clone(),
        output: None
    };
    let batch: TransactionBatch = inputs.batch;
    let mut tree: TornadoTree = inputs.tornado;

    // optional: calculate and mutate balance(s)
    // balances could be committed to the L1 or stored only in the L2
    for transfer in &batch.transfers {
        //todo: check teh transfer signature
    }

    // optional: calculate and mutate balance(s)
    for deposit in &batch.deposits{

    }
    // optional: calculate and mutate balance(s)
    for withdrawal in &batch.withdrawals{

    }

    // hash the batch and add it to the tornado tree
    let new_leaf: Vec<u8> = batch.hash();
    tree.add_leaf(new_leaf);
    env::commit(&tree);
}
