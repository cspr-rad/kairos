use casper_event_standard::Event;
use casper_types::Key;

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Mint {
    recipient: Key,
    token_id: String,
    data: String,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Burn {
    owner: Key,
    token_id: String,
    burner: Key,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Approval {
    owner: Key,
    spender: Key,
    token_id: String,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct ApprovalRevoked {
    owner: Key,
    token_id: String,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct ApprovalForAll {
    owner: Key,
    operator: Key,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct RevokedForAll {
    owner: Key,
    operator: Key,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Transfer {
    owner: Key,
    spender: Option<Key>,
    recipient: Key,
    token_id: String,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct MetadataUpdated {
    token_id: String,
    data: String,
}

#[derive(Event, Debug, PartialEq, Eq, Default)]
pub struct VariablesSet {}

#[derive(Event, Debug, PartialEq, Eq, Default)]
pub struct Migration {}
