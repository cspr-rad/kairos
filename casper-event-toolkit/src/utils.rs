use casper_types::contracts::NamedKeys;
use casper_types::{Key, URef};

use crate::error::ToolkitError;

pub fn extract_uref_from_named_keys(
    keys: &NamedKeys,
    key_name: &str,
) -> Result<URef, ToolkitError> {
    let key = keys
        .get(key_name)
        .ok_or_else(|| ToolkitError::MissingMetadataKey {
            context: key_name.to_string(),
        })?;

    let uref = match key {
        Key::URef(uref) => Ok(uref),
        _ => Err(ToolkitError::InvalidKeyType {
            context: key_name.to_string(),
        }),
    }?;

    Ok(*uref)
}

pub fn parse_hash(hash_str: &str) -> Result<[u8; 32], ToolkitError> {
    let bytes = hex::decode(hash_str).map_err(|_e| ToolkitError::InvalidHash {
        context: "hex parsing failed",
    })?;
    let hash = casper_types::HashAddr::try_from(bytes.as_ref()).map_err(|_e| {
        ToolkitError::InvalidHash {
            context: "not valid Casper address",
        }
    })?;

    Ok(hash)
}
