use crate::error::ReplicatorError;

use casper_client::types::Contract;

/// Transforms a contract's named keys into a proper `NamedKeys` (from `casper_types`).
///
pub fn extract_named_keys(contract: Contract) -> casper_types::contracts::NamedKeys {
    contract
        .named_keys()
        .map(|named_key| (named_key.name().to_owned(), named_key.key().unwrap()))
        .collect()
}
