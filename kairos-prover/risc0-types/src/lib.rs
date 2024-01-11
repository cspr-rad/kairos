#![no_std]
use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize)]
pub struct State{
    pub x: u32,
    pub y: u32
}