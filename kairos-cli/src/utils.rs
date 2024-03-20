use crate::error::CliError;

/// Custom parser function to convert a hexadecimal string to a byte array.
pub fn parse_hex_string(s: &str) -> Result<Vec<u8>, CliError> {
    hex::decode(s).map_err(|e| e.into())
}
