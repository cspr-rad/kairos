use casper_event_standard::Schemas;
use casper_types::bytesrepr::{FromBytes, ToBytes};

use crate::error::ReplicatorError;
use crate::event::Event;
use crate::metadata::CesMetadataRef;
use crate::rpc::client::CasperClient;

pub struct Fetcher {
    pub client: CasperClient,
    // Metdadata
    pub ces_metadata: CesMetadataRef,
}

impl Fetcher {
    pub async fn fetch_events_count(&self) -> Result<u32, ReplicatorError> {
        let events_length_uref = &self.ces_metadata.events_length;
        let events_length_value = self.client.get_stored_clvalue(&events_length_uref).await;
        let events_length: u32 = events_length_value
            .into_t()
            .map_err(|e| ReplicatorError::InvalidCLValueType(e.to_string()))?;

        Ok(events_length)
    }

    pub async fn fetch_schema(&self) -> Result<Schemas, ReplicatorError> {
        let events_schema_uref = &self.ces_metadata.events_schema;
        let events_schema_value = self.client.get_stored_clvalue(&events_schema_uref).await;
        let events_schema: Schemas = events_schema_value
            .into_t()
            .map_err(|e| ReplicatorError::InvalidCLValueType(e.to_string()))?;

        Ok(events_schema)
    }

    pub async fn fetch_event(
        &self,
        id: u64,
        event_schema: &Schemas,
    ) -> Result<Event, ReplicatorError> {
        let events_data_uref = &self.ces_metadata.events_data;
        let event_value = self
            .client
            .get_stored_clvalue_from_dict(&events_data_uref, &id.to_string())
            .await;

        let bytes = event_value.inner_bytes();
        let (_total_length, event_data) = u32::from_bytes(bytes).unwrap();
        let (event_name, _rem2a) = String::from_bytes(event_data).unwrap();
        let event_name = event_name.strip_prefix("event_").unwrap();

        // Parse dynamic event data.
        let dynamic_event_schema = match event_schema.0.get(event_name) {
            Some(schema) => Ok(schema.clone()),
            None => Err(ReplicatorError::MissingEventSchema(event_name.to_string())),
        }?;
        let dynamic_event =
            crate::event::parse_dynamic_event(dynamic_event_schema.to_vec(), &event_data);

        Ok(dynamic_event)
    }
}
