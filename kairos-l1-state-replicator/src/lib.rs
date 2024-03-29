use casper_event_standard::Schemas;
use casper_types::bytesrepr::{FromBytes, ToBytes};
use error::ReplicatorError;
use parser::EventParsed;
use rpc::CasperClient;

pub mod error;
pub mod parser;
pub mod rpc;
pub mod rpc_utils;
pub mod utils;

struct CesMetadataRef {
    pub events_schema: casper_types::URef,
    pub events_length: casper_types::URef,
    pub events_data: casper_types::URef,
}

const EVENTS_SCHEMA_KEY: &str = "__events_schema";
const EVENTS_LENGTH_KEY: &str = "__events_length";
const EVENTS_DATA_KEY: &str = "__events";

pub struct CasperStateReplicator {
    // Config state.
    contract_hash: String,
    client: CasperClient,
    // Dynamic state.
    ces_metadata_ref: Option<CesMetadataRef>,
    events_length: Option<u32>,
    events_schema: Option<Schemas>,
}

impl CasperStateReplicator {
    pub fn from_contract(client: CasperClient, contract_hash: &str) -> Self {
        Self {
            client,
            contract_hash: contract_hash.to_string(),
            ces_metadata_ref: None,
            events_length: None,
            events_schema: None,
        }
    }

    pub async fn fetch_metadata(&mut self) -> Result<(), ReplicatorError> {
        // Fetch contract named keys.
        let contract_named_keys = self
            .client
            .get_contract_named_keys(&self.contract_hash)
            .await;

        // Extract CES metadata from named keys.
        let events_schema_uref =
            utils::extract_uref_from_named_keys(&contract_named_keys, EVENTS_SCHEMA_KEY)?;
        let events_length_uref =
            utils::extract_uref_from_named_keys(&contract_named_keys, EVENTS_LENGTH_KEY)?;
        let events_data_uref =
            utils::extract_uref_from_named_keys(&contract_named_keys, EVENTS_DATA_KEY)?;

        self.ces_metadata_ref = Some(CesMetadataRef {
            events_data: events_data_uref,
            events_length: events_length_uref,
            events_schema: events_schema_uref,
        });

        Ok(())
    }

    pub async fn fetch_events_count(&mut self) -> Result<(), ReplicatorError> {
        let events_length_uref = match &self.ces_metadata_ref {
            Some(v) => &v.events_length,
            None => panic!("Metadata not loaded."), // TODO: Maybe load it automatically?
        };
        let events_length_value = self.client.get_stored_clvalue(&events_length_uref).await;
        let events_length: u32 = events_length_value
            .into_t()
            .map_err(|e| ReplicatorError::InvalidCLValueType(e.to_string()))?;
        self.events_length = Some(events_length);

        Ok(())
    }

    pub async fn fetch_schema(&mut self) {
        let events_schema_uref = match &self.ces_metadata_ref {
            Some(v) => &v.events_schema,
            None => panic!("Metadata not loaded."), // TODO: Maybe load it automatically?
        };
        let schema_value = self.client.get_stored_clvalue(&events_schema_uref).await;
        let schema_bytes = schema_value
            .to_bytes()
            .map_err(|_e| "Unable to get schema bytes.")
            .unwrap();
        let (schema_clvalue, remainder) = casper_types::CLValue::from_bytes(&schema_bytes)
            .map_err(|_e| "Unable to parse schema bytes.")
            .unwrap();
        assert!(remainder.len() == 0);

        let (events_schema2, rem) = Schemas::from_bytes(&schema_clvalue.inner_bytes()).unwrap();
        assert!(rem.len() == 0);
        self.events_schema = Some(events_schema2)
    }

    pub fn load_schema(&mut self, local_schemas: Schemas) {
        // Optional - schema validation.
        //let schema_clvalue = local_schemas.to_bytes().unwrap();
        //let chain_schema_bytes = schema_clvalue.inner_bytes().clone();
        //assert_eq!(chain_schema_bytes, local_schema_bytes);

        self.events_schema = Some(local_schemas);
    }

    pub async fn fetch_event(&mut self, id: u64) -> EventParsed {
        let events_data_uref = match &self.ces_metadata_ref {
            Some(v) => &v.events_data,
            None => panic!("Metadata not loaded."), // TODO: Maybe load it automatically?
        };
        let event_value = self
            .client
            .get_stored_clvalue_from_dict(&events_data_uref, &id.to_string())
            .await;

        let bytes = event_value.inner_bytes();
        let (_total_length, event_data) = u32::from_bytes(bytes).unwrap();
        let (event_name, _rem2a) = String::from_bytes(event_data).unwrap();
        let event_name = event_name.strip_prefix("event_").unwrap();

        // Parse dynamic event data.
        let dynamic_event_schema = match &self.events_schema {
            Some(schema) => schema.0.get(event_name).unwrap().clone(),
            None => panic!("Schema not loaded."), // TODO: Maybe load it automatically?
        };
        let dynamic_event = parser::parse_dynamic_event(dynamic_event_schema.to_vec(), &event_data);

        dynamic_event
    }

    //TODO: pub async fn get_events_count()
}
