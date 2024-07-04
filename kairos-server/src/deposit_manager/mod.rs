pub mod error;

use anyhow::{anyhow, Result};
use backoff::{future::retry, Error, ExponentialBackoff};
use casper_client::{
    get_state_root_hash, query_global_state, types::DeployHash, types::StoredValue, JsonRpcId,
    Verbosity,
};
use rand::Rng;
use reqwest::Url;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::RwLock;

use crate::state::BatchStateManager;
use casper_client_types::ContractHash;
use casper_event_standard::casper_types::{bytesrepr::FromBytes, Key};
use casper_event_toolkit::fetcher::{Fetcher, Schemas};
use casper_event_toolkit::metadata::CesMetadataRef;
use casper_event_toolkit::rpc::client::CasperClient;
use casper_event_toolkit::rpc::compat::key_to_client_types;
use contract_utils::constants::KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER;
use error::DepositManagerError;
use kairos_circuit_logic::transactions::{KairosTransaction, L1Deposit};

pub struct DepositManager {
    /// The last deposit event index that was added to the batch
    last_deposit_added_to_batch: AtomicU32,
    pub known_deposit_deploys: RwLock<HashSet<DeployHash>>,
    fetcher: Fetcher,
    schemas: Schemas,
}

impl DepositManager {
    pub async fn new(
        casper_rpc_url: &Url,
        contract_hash: &ContractHash,
    ) -> Result<Self, DepositManagerError> {
        tracing::info!("Initializing event manager");

        let casper_client = CasperClient::new(casper_rpc_url.as_str());

        let metadata = retry(ExponentialBackoff::default(), || async {
            CesMetadataRef::fetch_metadata(&casper_client, &contract_hash.to_string())
                .await
                .map_err(|err| {
                    tracing::info!("Failed to fetch metadata {}", err);
                    Error::transient(err)
                })
        })
        .await?;

        tracing::info!("Metadata fetched successfully");

        let fetcher = Fetcher {
            client: casper_client,
            ces_metadata: metadata,
        };

        let schemas = retry(ExponentialBackoff::default(), || async {
            fetcher.fetch_schema().await.map_err(|err| {
                tracing::info!("Failed to fetch schema {}", err);
                Error::transient(err)
            })
        })
        .await?;

        tracing::info!("Schemas fetched successfully");

        let last_processed_deposit_index: u32 =
            get_last_deposit_counter(casper_rpc_url, contract_hash)
                .await
                .map_err(|err| {
                    DepositManagerError::UnexpectedError(format!(
                        "Failed to fetch the index for the last processed deposit: {}",
                        err
                    ))
                })?;

        tracing::info!("On-chain stored last processed deposit index fetched successfully");

        Ok(DepositManager {
            last_deposit_added_to_batch: AtomicU32::new(last_processed_deposit_index + 1),
            fetcher,
            schemas,
            known_deposit_deploys: RwLock::new(HashSet::new()),
        })
    }

    /// Processes new events starting from the last known event ID.
    pub async fn add_new_events_to(
        &self,
        batch_state_manager: &BatchStateManager,
    ) -> Result<(), DepositManagerError> {
        let last_unprocessed_deposit_index = self.fetcher.fetch_events_count().await?;

        let next_deposit_index = self
            .last_deposit_added_to_batch
            .fetch_add(1, Ordering::SeqCst)
            + 1;
        assert!(next_deposit_index <= last_unprocessed_deposit_index);
        for deposit_index in next_deposit_index..=last_unprocessed_deposit_index {
            let untyped_event = self
                .fetcher
                .fetch_event(deposit_index, &self.schemas)
                .await?;

            // (koxu1996) NOTE: I think we should rather use full transaction data (ASN) for events,
            // parse them here with `kairos-tx` and then push to Data Availability layer.

            match untyped_event.name.as_str() {
                "Deposit" => {
                    let data = untyped_event.to_ces_bytes()?;
                    let (deposit, _) = L1Deposit::from_bytes(&data).map_err(|err| {
                        DepositManagerError::UnexpectedError(format!(
                            "Failed to parse deposit event from bytes: {}",
                            err
                        ))
                    })?;
                    batch_state_manager
                        .enqueue_transaction(KairosTransaction::Deposit(L1Deposit {
                            recipient: deposit.recipient,
                            amount: deposit.amount,
                        }))
                        .await
                        .map_err(|err| {
                            DepositManagerError::UnexpectedError(format!(
                                "Failed to parse deposit event from bytes: {}",
                                err
                            ))
                        })?;
                }
                other => {
                    tracing::error!("Unknown event type: {}", other)
                }
            }
        }
        self.last_deposit_added_to_batch
            .store(last_unprocessed_deposit_index, Ordering::SeqCst);
        Ok(())
    }
}

async fn get_last_deposit_counter(
    casper_node_rpc_url: &Url,
    contract_hash: &ContractHash,
) -> Result<u32> {
    let expected_rpc_id = JsonRpcId::Number(rand::thread_rng().gen::<i64>());
    let state_root_hash = get_state_root_hash(
        expected_rpc_id.clone(),
        casper_node_rpc_url.as_str(),
        Verbosity::High,
        Option::None,
    )
    .await
    .map_err(Into::<anyhow::Error>::into)
    .and_then(|response| {
        if response.id == expected_rpc_id {
            response
                .result
                .state_root_hash
                .ok_or(anyhow!("No state root hash present in response"))
        } else {
            Err(anyhow!("JSON RPC Id missmatch"))
        }
    })?;

    let expected_rpc_id = JsonRpcId::Number(rand::thread_rng().gen::<i64>());
    let contract_hash_key = key_to_client_types(&Key::Hash(contract_hash.value()))?;
    query_global_state(
        expected_rpc_id.clone(),
        casper_node_rpc_url.as_str(),
        Verbosity::High,
        casper_client::rpcs::GlobalStateIdentifier::StateRootHash(state_root_hash), // fetches recent blocks state root hash
        contract_hash_key,
        vec![KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER.to_string()],
    )
    .await
    .map_err(Into::<anyhow::Error>::into)
    .and_then(|response| {
        if response.id == expected_rpc_id {
            match response.result.stored_value {
                StoredValue::CLValue(last_deposit_counter) => last_deposit_counter
                    .into_t()
                    .map_err(|err| anyhow!("Failed to convert from CLValue to u32: {}", err)),
                _ => Err(anyhow!("Unexpected result type, type is not a CLValue")),
            }
        } else {
            Err(anyhow!("JSON RPC Id missmatch"))
        }
    })
}
