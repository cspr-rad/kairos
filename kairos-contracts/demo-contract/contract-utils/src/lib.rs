#![no_std]
pub mod constants;

use casper_event_standard::casper_types::Key;
use casper_event_standard::Event;

extern crate alloc;

use alloc::vec::Vec;

#[derive(Event)]
pub struct Deposit {
    pub depositor: Key, // TODO: Deprecate this.
    pub amount: u64,    // TODO: Deprecate this.
    pub tx: Vec<u8>,
}
