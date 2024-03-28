use std::collections::BTreeMap;

use casper_event_standard::{CLType2, Schemas};
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLValue,
};

use crate::rpc::CasperClient;

mod cep78_events;

mod rpc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch some deploy.
    let client = CasperClient::new_mainnet();
    let deploy_result = client
        .get_deploy("5fc34e15776a08bd059355acd57937b56ddc48ad0c2f55bd8d0b376170c8a412")
        .await;

    //println!("Deploy: {:?}", deploy_result);

    // Fetch contract details (correlated with depoly).
    let contract = client
        .get_contract("fe03021407879ce6fc5e035b70ff6a90941afdbea325a9164c7a497827efa7ff")
        .await;

    //println!("Contract: {:?}", contract);

    // Fetch latest state root hash.
    let state_root_hash = client.get_state_root_hash().await;

    //println!("State root hash: {:?}", state_root_hash);

    // Load contract metadata without schema.
    let mut events_schema_uref: Option<String> = None;
    let mut events_length_uref: Option<String> = None;
    let mut events_uref: Option<String> = None;
    for named_key in contract.named_keys() {
        if named_key.name() == "__events_schema" {
            events_schema_uref = Some(named_key.key().unwrap().to_formatted_string());
        }
        if named_key.name() == "__events_length" {
            events_length_uref = Some(named_key.key().unwrap().to_formatted_string());
        }
        if named_key.name() == "__events" {
            events_uref = Some(named_key.key().unwrap().to_formatted_string());
        }
    }
    let (events_schema_uref, events_length_uref, events_uref) =
        match (events_schema_uref, events_length_uref, events_uref) {
            (Some(events_schema_uref), Some(events_length_uref), Some(events_uref)) => {
                Ok((events_schema_uref, events_length_uref, events_uref))
            }
            _ => Err("Expected named keys."),
        }?;

    //println!("Events schema uref: {:?}", events_schema_uref);
    //println!("Events length uref: {:?}", events_length_uref);
    //println!("Events uref: {:?}", events_uref);

    // Load contract event schemas.
    let schema_value = client.get_stored_clvalue(&events_schema_uref).await;

    //println!("Events schema: {:?}", schema_value);

    // We cannot parse CLValue based on the CLType from the Argument raw data, as it may contain an Any type
    // which we do not know how to parse. Therefore, we should parse the raw bytes, ignore the clType field,
    // and provide the hardcoded CLType with the cltype.Dynamic type instead of Any
    let schema_bytes = schema_value
        .to_bytes()
        .map_err(|_e| "Unable to get schema bytes.")?;
    let (schema_clvalue, remainder) = casper_types::CLValue::from_bytes(&schema_bytes)
        .map_err(|_e| "Unable to parse schema bytes.")?;
    assert!(remainder.len() == 0);
    let events_schema: BTreeMap<String, Vec<(String, CLType2)>> =
        schema_clvalue.clone().into_t().unwrap();

    println!("Events schema parsed: {:?}", events_schema);

    // alternatively:
    let (events_schema2, rem) = Schemas::from_bytes(&schema_clvalue.inner_bytes()).unwrap();
    assert!(rem.len() == 0);
    //println!("Events schema parsed: {:?}", events_schema2);
    //println!("Incomplete Burn schema (without `burner`): {:?}", events_schema2.0.get("Burn"));

    // User locally defined schemas.
    let local_schema = Schemas::new()
        .with::<cep78_events::Mint>()
        .with::<cep78_events::Burn>()
        .with::<cep78_events::Approval>()
        .with::<cep78_events::ApprovalRevoked>()
        .with::<cep78_events::ApprovalForAll>()
        .with::<cep78_events::Transfer>()
        .with::<cep78_events::MetadataUpdated>()
        .with::<cep78_events::VariablesSet>()
        .with::<cep78_events::Migration>();
    let local_schema_bytes = local_schema.to_bytes().unwrap();

    // Optional - schema validation.
    //let chain_schema_bytes = schema_clvalue.inner_bytes().clone();
    //assert_eq!(chain_schema_bytes, local_schema_bytes);

    // Load contract events length.
    let events_length_value = client.get_stored_clvalue(&events_length_uref).await;
    let events_length: u32 = events_length_value.into_t().unwrap();

    println!("Events length: {:?}", events_length);

    // Fetch each event data.
    for event_id in 0..events_length {
        let event_value = client
            .get_stored_clvalue_from_dict(&events_uref, &event_id.to_string())
            .await;
        // println!("Event {:?}: {:?}", event_id, event_value);

        let bytes = event_value.inner_bytes();
        let (_total_length, event_data) = u32::from_bytes(bytes).unwrap();
        let (event_name, _rem2a) = String::from_bytes(event_data).unwrap();
        let event_name = event_name.strip_prefix("event_").unwrap();
        println!("Event name: {:?}", event_name);

        // Parse dynamic event data.
        let dynamic_event_schema = events_schema.get(event_name).unwrap().clone();
        let dynamic_event = parse_dynamic_event(dynamic_event_schema, &event_data);
        println!("Event data parsed dynamically: {:?}", dynamic_event);

        match dynamic_event.name.as_str() {
            "Mint" => {
                let data = dynamic_event.to_ces_bytes();
                let (parsed_further, rem) = cep78_events::Mint::from_bytes(&data).unwrap();
                assert!(rem.len() == 0);
                println!("Event data parsed statically: {:?}", parsed_further);
            }
            other => {
                println!("Unknown event type: {}", other)
            }
        }

        break;
    }

    Ok(())
}

#[derive(Debug)]
struct EventParsed {
    pub name: String,
    pub fields: Vec<(String, CLValue)>,
}

impl EventParsed {
    fn to_ces_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = vec![];

        let prefixed_name = String::from(EVENT_PREFIX) + &self.name;
        let event_name = String::to_bytes(&prefixed_name).unwrap();
        result.extend_from_slice(&event_name);

        for (_field_name, field_value) in &self.fields {
            let field_bytes = field_value.inner_bytes();
            result.extend_from_slice(field_bytes);
        }

        result
    }
}

const EVENT_PREFIX: &str = "event_";

fn parse_dynamic_event(
    dynamic_event_schema: Vec<(String, CLType2)>,
    event_data: &[u8],
) -> EventParsed {
    let (event_name, mut remainder) = String::from_bytes(event_data).unwrap();
    let event_name = event_name.strip_prefix(EVENT_PREFIX).unwrap();
    let mut event_fields = vec![];
    for (field_name, field_type) in dynamic_event_schema {
        let field_value: CLValue = match field_type.downcast() {
            casper_types::CLType::Bool => todo!(),
            casper_types::CLType::I32 => todo!(),
            casper_types::CLType::I64 => todo!(),
            casper_types::CLType::U8 => todo!(),
            casper_types::CLType::U32 => todo!(),
            casper_types::CLType::U64 => todo!(),
            casper_types::CLType::U128 => todo!(),
            casper_types::CLType::U256 => todo!(),
            casper_types::CLType::U512 => todo!(),
            casper_types::CLType::Unit => todo!(),
            casper_types::CLType::String => {
                let (value, new_remainder) = String::from_bytes(remainder).unwrap();
                remainder = new_remainder;
                let value_bytes = value.to_bytes().unwrap();
                CLValue::from_components(casper_types::CLType::String, value_bytes)
            }
            casper_types::CLType::Key => {
                let (value, new_remainder) = casper_types::Key::from_bytes(remainder).unwrap();
                remainder = new_remainder;
                let value_bytes = value.to_bytes().unwrap();
                CLValue::from_components(casper_types::CLType::Key, value_bytes)
            }
            casper_types::CLType::URef => todo!(),
            casper_types::CLType::PublicKey => todo!(),
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
        event_fields.push((field_name, field_value));
    }

    EventParsed {
        name: event_name.into(),
        fields: event_fields,
    }
}