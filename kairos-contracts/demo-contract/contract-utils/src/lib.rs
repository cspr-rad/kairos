#![no_std]
pub mod constants;

use casper_event_standard::casper_types::Key;
use casper_event_standard::Event;

extern crate alloc;

#[derive(Event)]
pub struct Deposit {
    pub depositor: Key,
    pub amount: u64,
}
