use std::sync::Arc;

use casper_event_toolkit::casper_types::bytesrepr::FromBytes;
use casper_event_toolkit::fetcher::{Fetcher, Schemas};
use casper_event_toolkit::metadata::CesMetadataRef;
use casper_event_toolkit::rpc::client::CasperClient;

use crate::state::ServerStateInner;
use kairos_circuit_logic::transactions::{KairosTransaction, L1Deposit};

use super::error::L1SyncError;

pub struct EventManager {
    next_event_id: u32,
    fetcher: Fetcher,
    schemas: Schemas,
    server_state: Arc<ServerStateInner>,
}

impl EventManager {
    pub async fn new(server_state: Arc<ServerStateInner>) -> Result<Self, L1SyncError> {
        tracing::info!("Initializing event manager");

        let rpc_url = server_state.server_config.casper_rpc.as_str();
        let contract_hash = server_state
            .server_config
            .kairos_demo_contract_hash
            .to_string();
        let client = CasperClient::new(rpc_url);
        let metadata = CesMetadataRef::fetch_metadata(&client, &contract_hash).await?;
        tracing::debug!("Metadata fetched successfully");

        let fetcher = Fetcher {
            client,
            ces_metadata: metadata,
        };
        let schemas = fetcher.fetch_schema().await?;
        tracing::debug!("Schemas fetched successfully");

        Ok(EventManager {
            next_event_id: 0,
            fetcher,
            schemas,
            server_state,
        })
    }

    /// Processes new events starting from the last known event ID.
    pub async fn process_new_events(&mut self) -> Result<(), L1SyncError> {
        tracing::info!("Looking for new events");

        let num_events = self.fetcher.fetch_events_count().await?;
        for i in self.next_event_id..num_events {
            let event = self.fetcher.fetch_event(i, &self.schemas).await?;
            tracing::debug!("Event {} fetched: {:?}.", i, event);

            let event_bytes = event.to_ces_bytes()?;

            // (koxu1996) NOTE: I think we should rather use full transaction data (ASN) for events,
            // parse them here with `kairos-tx` and then push to Data Availability layer.

            match event.name.as_str() {
                "L1Deposit" => {
                    // Parse simplified deposit data.
                    let (deposit, _) = L1Deposit::from_bytes(&event_bytes)
                        .expect("Failed to parse deposit event from bytes");

                    let amount = deposit.amount;
                    let recipient: Vec<u8> = deposit.recipient;
                    let txn = KairosTransaction::Deposit(L1Deposit { amount, recipient });

                    // Push deposit to trie.
                    self.server_state
                        .batch_state_manager
                        .enqueue_transaction(txn)
                        .await
                        .map_err(|e| {
                            L1SyncError::UnexpectedError(format!("unable to batch tx: {}", e))
                        })?;
                }
                name => {
                    tracing::error!("Unrecognized event {}", name);
                }
            }

            self.next_event_id = i + 1;
        }

        Ok(())
    }
}
