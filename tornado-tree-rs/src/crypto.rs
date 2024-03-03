extern crate sha256;
use sha2::{Sha256, Digest};

pub fn hash_bytes(input: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    result.to_vec()
}

pub fn hash_left_right(mut left: Vec<u8>, mut right: Vec<u8>) -> Vec<u8>{
    left.append(&mut right);
    hash_bytes(left)
}