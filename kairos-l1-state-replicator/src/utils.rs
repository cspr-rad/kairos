use casper_types::{contracts::NamedKeys, Key, URef};

use crate::error::ReplicatorError;

pub fn extract_uref_from_named_keys(
    keys: &NamedKeys,
    key_name: &str,
) -> Result<URef, ReplicatorError> {
    let key = keys
        .get(key_name)
        .ok_or_else(|| ReplicatorError::MissingMetadataKey {
            context: key_name.to_string(),
        })?;

    let uref = match key {
        Key::URef(uref) => Ok(uref),
        _ => Err(ReplicatorError::InvalidKeyType {
            context: key_name.to_string(),
        }),
    }?;

    Ok(uref.clone())
}
