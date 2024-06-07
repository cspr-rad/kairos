use std::sync::Arc;

use casper_event_toolkit::fetcher::{Fetcher, Schemas};
use casper_event_toolkit::metadata::CesMetadataRef;
use casper_event_toolkit::rpc::client::CasperClient;

use crate::state::ServerStateInner;
use kairos_circuit_logic::transactions::{KairosTransaction, L1Deposit};

use super::error::L1SyncError;

pub struct EventManager {
    next_event_id: u32,
    fetcher: Option<Fetcher>,
    schemas: Option<Schemas>,
    server_state: Arc<ServerStateInner>,
}

impl EventManager {
    pub fn new(server_state: Arc<ServerStateInner>) -> Self {
        EventManager {
            next_event_id: 0,
            fetcher: None,
            schemas: None,
            server_state,
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

        // Ensure fetcher and schemas are initialized
        let fetcher = self.fetcher.as_ref().ok_or_else(|| {
            L1SyncError::InitializationError("Fetcher not initialized".to_string())
        })?;
        let schemas = self.schemas.as_ref().ok_or_else(|| {
            L1SyncError::InitializationError("Schemas not initialized".to_string())
        })?;

        let num_events = fetcher.fetch_events_count().await?;
        for i in self.next_event_id..num_events {
            let event = fetcher.fetch_event(i, schemas).await?;
            tracing::debug!("Event {} fetched: {:?}.", i, event);

            // TODO: Parse full transaction data from event, then push it to Data Availability layer.

            // TODO: Once we have ASN transaction, it should be converted and pushed into batch.
            let recipient: Vec<u8> = "cafebabe".into();
            let txn = KairosTransaction::Deposit(L1Deposit {
                amount: 100,
                recipient,
            });
            self.server_state
                .batch_state_manager
                .enqueue_transaction(txn)
                .await
                .map_err(|e| L1SyncError::UnexpectedError(format!("unable to batch tx: {}", e)))?;
            self.next_event_id = i + 1;
        }

        Ok(())
    }
}
