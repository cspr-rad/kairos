use crate::error::ReplicatorError;
use crate::rpc::client::CasperClient;
use crate::utils;

const EVENTS_SCHEMA_KEY: &str = "__events_schema";
const EVENTS_LENGTH_KEY: &str = "__events_length";
const EVENTS_DATA_KEY: &str = "__events";

#[derive(Clone)]
pub struct CesMetadataRef {
    pub events_schema: casper_types::URef,
    pub events_length: casper_types::URef,
    pub events_data: casper_types::URef,
}

impl CesMetadataRef {
    pub async fn fetch_metadata(
        client: &CasperClient,
        contract_hash: &str,
    ) -> Result<CesMetadataRef, ReplicatorError> {
        // Fetch contract named keys.
        let contract_named_keys = client.get_contract_named_keys(contract_hash).await?;

        // Extract CES metadata from named keys.
        let events_schema_uref =
            utils::extract_uref_from_named_keys(&contract_named_keys, EVENTS_SCHEMA_KEY)?;
        let events_length_uref =
            utils::extract_uref_from_named_keys(&contract_named_keys, EVENTS_LENGTH_KEY)?;
        let events_data_uref =
            utils::extract_uref_from_named_keys(&contract_named_keys, EVENTS_DATA_KEY)?;

        Ok(CesMetadataRef {
            events_data: events_data_uref,
            events_length: events_length_uref,
            events_schema: events_schema_uref,
        })
    }
}
