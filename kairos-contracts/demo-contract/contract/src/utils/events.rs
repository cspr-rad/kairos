use casper_event_standard::Event;
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use casper_types::Key;

#[derive(Event)]
pub struct Deposit {
    pub account: Key,
    pub amount: u64,
    pub timestamp: Option<String>,
    pub tx: Vec<u8>,
}
