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
