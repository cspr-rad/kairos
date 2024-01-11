#![no_std]
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct State {
    pub x: u32,
    pub y: u32,
}
