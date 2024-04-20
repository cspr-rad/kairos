use casper_event_standard::Schema;
use casper_types::{
    bytesrepr::{FromBytes, ToBytes}, CLType, CLValue
};

use crate::error::ToolkitError;

#[derive(Debug)]
pub struct Event {
    pub name: String,
    pub fields: Vec<(String, CLValue)>,
}

impl Event {
    pub fn to_ces_bytes(&self) -> Result<Vec<u8>, ToolkitError> {
        let mut result: Vec<u8> = vec![];

        let prefixed_name = String::from(EVENT_PREFIX) + &self.name;
        let event_name =
            String::to_bytes(&prefixed_name).map_err(|_e| ToolkitError::SerializationError {
                context: "event_name",
            })?;
        result.extend_from_slice(&event_name);

        for (_field_name, field_value) in &self.fields {
            let field_bytes = field_value.inner_bytes();
            result.extend_from_slice(field_bytes);
        }

        Ok(result)
    }
}

const EVENT_PREFIX: &str = "event_";

pub fn parse_dynamic_event_data(
    dynamic_event_schema: Schema,
    event_data: &[u8],
) -> Vec<(String, CLValue)> {
    let mut event_fields = vec![];

    let mut remainder = event_data;
    let schema_fields = dynamic_event_schema.to_vec();
    for (field_name, field_type) in schema_fields {

        let cltype = field_type.downcast();
        let (field_value, new_remainder) = parse_dynamic_clvalue(&cltype, remainder).unwrap();
        remainder = new_remainder;

        event_fields.push((field_name, field_value));
    }

    event_fields
}

pub fn parse_dynamic_clvalue<'a>(cltype: &CLType, bytes: &'a [u8]) -> Result<(CLValue, &'a [u8]), ToolkitError> {
    let result = match cltype {
        casper_types::CLType::Bool => {
            let (value, new_remainder) = bool::from_bytes(bytes).unwrap();
            let value_bytes = value.to_bytes().unwrap();
            (CLValue::from_components(casper_types::CLType::Bool, value_bytes), new_remainder)
        },
        casper_types::CLType::I32 => {
            let (value, new_remainder) = i32::from_bytes(bytes).unwrap();
            let value_bytes = value.to_bytes().unwrap();
            (CLValue::from_components(casper_types::CLType::I32, value_bytes), new_remainder)
        },
        casper_types::CLType::I64 => {
            let (value, new_remainder) = i64::from_bytes(bytes).unwrap();
            let value_bytes = value.to_bytes().unwrap();
            (CLValue::from_components(casper_types::CLType::I64, value_bytes), new_remainder)
        },
        casper_types::CLType::U8 => {
            let (value, new_remainder) = u8::from_bytes(bytes).unwrap();
            let value_bytes = value.to_bytes().unwrap();
            (CLValue::from_components(casper_types::CLType::U8, value_bytes), new_remainder)
        },
        casper_types::CLType::U32 => {
            let (value, new_remainder) = u32::from_bytes(bytes).unwrap();
            let value_bytes = value.to_bytes().unwrap();
            (CLValue::from_components(casper_types::CLType::U32, value_bytes), new_remainder)
        },
        casper_types::CLType::U64 => {
            let (value, new_remainder) = u64::from_bytes(bytes).unwrap();
            let value_bytes = value.to_bytes().unwrap();
            (CLValue::from_components(casper_types::CLType::U64, value_bytes), new_remainder)
        },
        casper_types::CLType::U128 => {
            let (value, new_remainder) = casper_types::U128::from_bytes(bytes).unwrap();
            let value_bytes = value.to_bytes().unwrap();
            (CLValue::from_components(casper_types::CLType::U128, value_bytes), new_remainder)
        },
        casper_types::CLType::U256 => {
            let (value, new_remainder) = casper_types::U256::from_bytes(bytes).unwrap();
            let value_bytes = value.to_bytes().unwrap();
            (CLValue::from_components(casper_types::CLType::U256, value_bytes), new_remainder)
        },
        casper_types::CLType::U512 => {
            let (value, new_remainder) = casper_types::U512::from_bytes(bytes).unwrap();
            let value_bytes = value.to_bytes().unwrap();
            (CLValue::from_components(casper_types::CLType::U512, value_bytes), new_remainder)
        },
        casper_types::CLType::Unit => {
            let (value, new_remainder) = <()>::from_bytes(bytes).unwrap();
            let value_bytes = value.to_bytes().unwrap();
            (CLValue::from_components(casper_types::CLType::Unit, value_bytes), new_remainder)
        },
        casper_types::CLType::String => {
            let (value, new_remainder) = String::from_bytes(bytes).unwrap();
            let value_bytes = value.to_bytes().unwrap();
            (CLValue::from_components(casper_types::CLType::String, value_bytes), new_remainder)
        }
        casper_types::CLType::Key => {
            let (value, new_remainder) = casper_types::Key::from_bytes(bytes).unwrap();
            let value_bytes = value.to_bytes().unwrap();
            (CLValue::from_components(casper_types::CLType::Key, value_bytes), new_remainder)
        }
        casper_types::CLType::URef => {
            let (value, new_remainder) = casper_types::URef::from_bytes(bytes).unwrap();
            let value_bytes = value.to_bytes().unwrap();
            (CLValue::from_components(casper_types::CLType::URef, value_bytes), new_remainder)
        },
        casper_types::CLType::PublicKey => {
            let (value, new_remainder) = casper_types::PublicKey::from_bytes(bytes).unwrap();
            let value_bytes = value.to_bytes().unwrap();
            (CLValue::from_components(casper_types::CLType::PublicKey, value_bytes), new_remainder)
        },
        casper_types::CLType::Option(_) => todo!(),
        casper_types::CLType::List(_) => todo!(),
        casper_types::CLType::ByteArray(_) => todo!(),
        casper_types::CLType::Result { ok, err } => todo!(),
        casper_types::CLType::Map { key, value } => todo!(),
        casper_types::CLType::Tuple1(_) => todo!(),
        casper_types::CLType::Tuple2(_) => todo!(),
        casper_types::CLType::Tuple3(_) => todo!(),
        casper_types::CLType::Any => todo!(),
    };
    
    Ok(result)
}
