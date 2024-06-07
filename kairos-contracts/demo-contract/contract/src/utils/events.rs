use casper_event_standard::Event;
extern crate alloc;
use casper_types::Key;

#[derive(Event)]
pub struct Deposit {
    pub depositor: Key,
    pub amount: u64,
}
