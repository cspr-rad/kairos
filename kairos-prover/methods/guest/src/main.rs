#![no_main]
#![no_std]  
use risc0_zkvm::guest::env;
use serde_json;
risc0_zkvm::guest::entry!(main);
use risc0_types::State;

pub fn main() {
    let x: u32 = env::read();
    let y: u32 = env::read();
    if x != y {
        panic!("X != Y");
    }
    let state = State{
        x,
        y
    };
    let data = &serde_json::to_vec(&state).unwrap();
    env::commit(data);
}
