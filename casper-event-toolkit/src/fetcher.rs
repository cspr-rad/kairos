use casper_event_standard::Schemas;
use casper_types::bytesrepr::FromBytes;

use crate::error::ToolkitError;
use crate::event::Event;
use crate::metadata::CesMetadataRef;
use crate::parser::{parse_event, parse_raw_event_name_and_data};
use crate::rpc::client::CasperClient;

pub struct Fetcher {
    pub client: CasperClient,
    // Metdadata
    pub ces_metadata: CesMetadataRef,
}

impl Fetcher {
    pub async fn fetch_events_count(&self) -> Result<u32, ToolkitError> {
        let events_length_uref = &self.ces_metadata.events_length;
        let events_length_value = self.client.get_stored_clvalue(events_length_uref).await?;
        let events_length: u32 = events_length_value
            .into_t()
            .map_err(|e| ToolkitError::InvalidCLValue(e.to_string()))?;

        Ok(events_length)
    }

    pub async fn fetch_schema(&self) -> Result<Schemas, ToolkitError> {
        let events_schema_uref = &self.ces_metadata.events_schema;
        let events_schema_value = self.client.get_stored_clvalue(events_schema_uref).await?;
        let events_schema: Schemas = events_schema_value
            .into_t()
            .map_err(|e| ToolkitError::InvalidCLValue(e.to_string()))?;

        Ok(events_schema)
    }

    pub async fn fetch_event(
        &self,
        id: u64,
        event_schema: &Schemas,
    ) -> Result<Event, ToolkitError> {
        let events_data_uref = &self.ces_metadata.events_data;
        let event_value = self
            .client
            .get_stored_clvalue_from_dict(events_data_uref, &id.to_string())
            .await?;
        let event_value_bytes = event_value.inner_bytes();
        let (event_name, event_data) = parse_raw_event_name_and_data(event_value_bytes)?;

        // Parse dynamic event data.
        let dynamic_event = parse_event(event_name, &event_data, event_schema)?;

        Ok(dynamic_event)
    }

    pub async fn fetch_events_from_deploy(
        &self,
        deploy_hash: &str,
        event_schema: &Schemas,
    ) -> Result<Vec<Event>, ToolkitError> {
        // Build deploy hash.
        let contract_hash_bytes = hex::decode(deploy_hash).unwrap();
        let contract_hash_bytes: [u8; 32] = contract_hash_bytes.try_into().unwrap();
        let deploy_hash = casper_client::types::DeployHash::new(contract_hash_bytes.into());

        let execution_result = self.client.get_deploy_result(deploy_hash).await?;
        let effects = match execution_result {
            casper_types::ExecutionResult::Failure { .. } => Err(ToolkitError::FailedDeployError),
            casper_types::ExecutionResult::Success { effect, .. } => Ok(effect),
        }?;

        let mut events = vec![];

        for entry in effects.transforms {
            // Look for data writes into the global state.
            let casper_types::Transform::WriteCLValue(clvalue) = entry.transform else {
                continue;
            };

            // Look specifically for dictionaries writes.
            const DICTIONARY_PREFIX: &str = "dictionary-";
            if !entry.key.starts_with(DICTIONARY_PREFIX) {
                continue;
            }

            // Try parsing CES value, but ignore errors - we don't really know if this is CES dictionary,
            // because write address is based on key (event ID).
            let Ok((_total_length, event_value_bytes)) = u32::from_bytes(clvalue.inner_bytes())
            else {
                continue;
            };
            let Ok((event_name, event_data)) = parse_raw_event_name_and_data(event_value_bytes)
            else {
                continue;
            };

            // Parse dynamic event data.
            let dynamic_event = parse_event(event_name, &event_data, event_schema)?;

            events.push(dynamic_event);
        }

        Ok(events)
    }
}
