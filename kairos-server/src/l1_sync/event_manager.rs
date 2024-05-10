use std::sync::Arc;

use casper_event_toolkit::fetcher::{Fetcher, Schemas};
use casper_event_toolkit::metadata::CesMetadataRef;
use casper_event_toolkit::rpc::client::CasperClient;

use crate::state::transactions::{Deposit, Signed, Transaction};
use crate::state::BatchStateManager;

use super::error::L1SyncError;

pub struct EventManager {
    next_event_id: u32,
    fetcher: Option<Fetcher>,
    schemas: Option<Schemas>,
    batch_service: Arc<BatchStateManager>,
}

impl EventManager {
    pub fn new(batch_service: Arc<BatchStateManager>) -> Self {
        EventManager {
            next_event_id: 0,
            fetcher: None,
            schemas: None,
            batch_service,
        }
    }

    /// Initializes state by building CES fetcher and obtaining schemas.
    pub async fn initialize(
        &mut self,
        rpc_url: &str,
        contract_hash: &str,
    ) -> Result<(), L1SyncError> {
        tracing::info!("Initializing event manager");

        let client = CasperClient::new(rpc_url);
        let metadata = CesMetadataRef::fetch_metadata(&client, contract_hash).await?;
        tracing::debug!("Metadata fetched successfully");

        let fetcher = Fetcher {
            client,
            ces_metadata: metadata,
        };
        let schemas = fetcher.fetch_schema().await?;
        tracing::debug!("Schemas fetched successfully");

        self.fetcher = Some(fetcher);
        self.schemas = Some(schemas);

        Ok(())
    }

    /// Processes new events starting from the last known event ID.
    pub async fn process_new_events(&mut self) -> Result<(), L1SyncError> {
        tracing::info!("Looking for new events");

        if let Some(fetcher) = &self.fetcher {
            let schemas = self.schemas.as_ref().unwrap(); // Assuming schemas are already loaded
            let num_events = fetcher.fetch_events_count().await?;
            for i in self.next_event_id..num_events {
                let event = fetcher.fetch_event(i.into(), schemas).await?;
                tracing::debug!("Event {} fetched: {:?}.", i, event);

                // TODO: Parse full transaction data from event, then push it to Data Availability layer.

                // TODO: Once we have ASN transaction, it should be converted and pushed into batch.
                let txn = Signed {
                    public_key: "cafebabe".into(),
                    nonce: 0,
                    transaction: Transaction::Deposit(Deposit { amount: 100 }),
                };
                self.batch_service
                    .enqueue_transaction(txn)
                    .await
                    .map_err(|e| {
                        L1SyncError::UnexpectedError(format!("unable to batch tx: {}", e))
                    })?;
                self.next_event_id = i + 1;
            }
        }

        Ok(())
    }
}
