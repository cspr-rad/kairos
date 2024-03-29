/// Schemas for CEP-78.
/// Source: https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/7815f090b51b9153dd33a3d7c0ab939b61e5a411/contract/src/utils.rs#L808-L821
///
use casper_event_standard::Schemas;

use crate::cep78::events;

pub fn get_local_schemas() -> Schemas {
    Schemas::new()
        .with::<events::Mint>()
        .with::<events::Burn>()
        .with::<events::Approval>()
        .with::<events::ApprovalRevoked>()
        .with::<events::ApprovalForAll>()
        .with::<events::Transfer>()
        .with::<events::MetadataUpdated>()
        .with::<events::VariablesSet>()
        .with::<events::Migration>()
}
