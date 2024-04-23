use casper_event_standard::Event;
extern crate alloc;
use alloc::string::String;
use casper_types::{Key, U512};

#[derive(Event)]
pub struct Deposit {
    pub account: Key,
    pub amount: U512,
    pub timestamp: Option<String>,
}
