use casper_event_standard::Event;
use casper_types::Key;

#[derive(Event, Debug)]
pub struct Mint {
    recipient: Key,
    token_id: String,
    data: String,
}

#[derive(Event, Debug)]
pub struct Burn {
    owner: Key,
    token_id: String,
    burner: Key,
}

#[derive(Event, Debug)]
pub struct Approval {
    owner: Key,
    spender: Key,
    token_id: String,
}

#[derive(Event, Debug)]
pub struct ApprovalRevoked {
    owner: Key,
    token_id: String,
}

#[derive(Event, Debug)]
pub struct ApprovalForAll {
    owner: Key,
    operator: Key,
}

#[derive(Event, Debug)]
pub struct RevokedForAll {
    owner: Key,
    operator: Key,
}

#[derive(Event, Debug)]
pub struct Transfer {
    owner: Key,
    spender: Option<Key>,
    recipient: Key,
    token_id: String,
}

#[derive(Event, Debug)]
pub struct MetadataUpdated {
    token_id: String,
    data: String,
}

#[derive(Event, Debug)]
pub struct VariablesSet {}

#[derive(Event, Debug)]
pub struct Migration {}
