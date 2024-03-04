pub mod crypto;
use crypto::hash_left_right;

use crate::crypto::hash_bytes;

use serde::{Serialize, Deserialize};

pub const ROOT_HISTORY_SIZE: u16 = 30u16;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TornadoTree{
    pub zero_node: Vec<u8>,
    pub zero_levels: Vec<Vec<u8>>,
    pub filled: Vec<Vec<u8>>,
    pub index: usize,
    pub depth: usize
}

impl TornadoTree{
    pub fn calculate_zero_levels(&mut self){
        let mut zero_levels: Vec<Vec<u8>> = vec![self.zero_node.clone()];
        for i in 0..self.depth - 1{
            zero_levels.push(hash_left_right(zero_levels[zero_levels.len()-1].clone(), zero_levels[zero_levels.len()-1].clone()));
        };
        self.zero_levels = zero_levels;
    }
    pub fn add_leaf(&mut self, leaf: Vec<u8>) {
        let mut current_index = self.index;
        let mut current_hash: Vec<u8> = leaf.clone();
        for i in 0..self.depth {
            if current_index % 2 == 0 {
                self.filled[i] = current_hash.clone();
                current_hash = hash_left_right(current_hash, self.zero_levels[i].clone());
            } else {
                let left = self.filled[i].clone();
                current_hash = hash_left_right(left.clone(), current_hash.clone());
            }
            current_index /= 2;
        }
        
        let current_root: Vec<u8> = self.filled.clone().pop().unwrap(); 
        self.index += 1;
    }
}

#[test]
fn test_tree(){
    // construct merkle tree
    let mut tree: TornadoTree = TornadoTree{
        zero_node: hash_bytes(vec![0;32]),
        zero_levels: Vec::new(),
        filled: vec![vec![], vec![], vec![], vec![], vec![]],
        index: 0,
        depth: 5
    };
    tree.calculate_zero_levels();
    let _ = tree.add_leaf(vec![242, 69, 81, 38, 252, 95, 197, 129, 177, 105, 42, 137, 129, 73, 125, 148, 130, 204, 83, 82, 126, 104, 106, 71, 156, 96, 55, 233, 132, 103, 128, 11]);
    let merkle_root = &tree.filled.pop().unwrap();
    // true -> right, false -> left
    println!("Merkle root: {:?}", &merkle_root);
}