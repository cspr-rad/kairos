use anyhow::Result;
use backoff::{future::retry, Error, ExponentialBackoff};
use casper_event_toolkit::fetcher::{Fetcher, Schemas};
use casper_event_toolkit::metadata::CesMetadataRef;
use casper_event_toolkit::rpc::client::CasperClient;
use casper_types::ContractHash;
use reqwest::Url;

pub struct EventManager {
    pub fetcher: Fetcher,
    pub schemas: Schemas,
}

impl EventManager {
    pub async fn new(casper_rpc_url: &Url, contract_hash: &ContractHash) -> Result<Self> {
        tracing::info!("Initializing event manager");
        let casper_client = CasperClient::new(casper_rpc_url.as_str());
        let metadata = retry(ExponentialBackoff::default(), || async {
            CesMetadataRef::fetch_metadata(&casper_client, &contract_hash.to_string())
                .await
                .map_err(Error::transient)
        })
        .await
        .expect("Failed to fetch the demo contracts event metadata");

        tracing::info!("Metadata fetched successfully");

        let fetcher = Fetcher {
            client: casper_client,
            ces_metadata: metadata,
        };

        let schemas = retry(ExponentialBackoff::default(), || async {
            fetcher.fetch_schema().await.map_err(Error::transient)
        })
        .await
        .expect("Failed to fetch the demo contracts event schema");

        tracing::info!("Schemas fetched successfully");

        Ok(EventManager { fetcher, schemas })
    }

    /// Processes new events starting from the last known event ID.
    pub async fn process_new_events(&self) {}
}
