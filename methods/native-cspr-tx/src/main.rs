#![no_main]
use risc0_zkvm::guest::env;
use tornado_tree_rs::TornadoTree;
risc0_zkvm::guest::entry!(main);

pub fn main() {
    let mut tree: TornadoTree = env::read();
    // add dummy leaf
    tree.add_leaf(vec![242, 69, 81, 38, 252, 95, 197, 129, 177, 105, 42, 137, 129, 73, 125, 148, 130, 204, 83, 82, 126, 104, 106, 71, 156, 96, 55, 233, 132, 103, 128, 11]);
    let merkle_root = &tree.filled.pop().unwrap();
    env::commit(&merkle_root);
}
