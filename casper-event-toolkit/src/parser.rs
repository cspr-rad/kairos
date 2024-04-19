use casper_types::bytesrepr::FromBytes;

use crate::error::ToolkitError;

pub fn parse_event_name_and_data(bytes: &[u8]) -> Result<(String, Vec<u8>), ToolkitError> {
    let (_total_length, event_data) =
        u32::from_bytes(bytes).map_err(|_e| ToolkitError::ParsingError {
            context: "event data length",
        })?;
    let (event_name, _rem2a) =
        String::from_bytes(event_data).map_err(|_e| ToolkitError::ParsingError {
            context: "event name",
        })?;
    let event_name =
        event_name
            .strip_prefix("event_")
            .ok_or_else(|| ToolkitError::ParsingError {
                context: "event prefix",
            })?;

    Ok((event_name.to_string(), event_data.to_vec()))
}
