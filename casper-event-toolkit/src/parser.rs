use casper_event_standard::Schemas;
use casper_types::bytesrepr::FromBytes;

use crate::error::ToolkitError;
use crate::event::Event;

pub fn parse_raw_event_name_and_data(bytes: &[u8]) -> Result<(String, Vec<u8>), ToolkitError> {
    let (_total_length, event_data_with_name) =
        u32::from_bytes(bytes).map_err(|_e| ToolkitError::ParsingError {
            context: "event data length",
        })?;
    let (event_name, event_data) =
        String::from_bytes(event_data_with_name).map_err(|_e| ToolkitError::ParsingError {
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

// Parse dynamic event data according to schema.
pub fn parse_event(event_name: String, event_data: &[u8], schemas: &Schemas) -> Result<Event, ToolkitError>{
    let dynamic_event_schema = match schemas.0.get(&event_name) {
        Some(schema) => Ok(schema.clone()),
        None => Err(ToolkitError::MissingEventSchema(event_name.to_string())),
    }?;
    let dynamic_event_data =
        crate::event::parse_dynamic_event_data(dynamic_event_schema.to_vec(), event_data);
    let dynamic_event = Event {
        name: event_name.to_string(),
        fields: dynamic_event_data,
    };

    Ok(dynamic_event)
}
