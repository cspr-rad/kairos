use crate::state::BatchStateManager;

use anyhow::{anyhow, Result};
use rand::Rng;
use reqwest::Url;

use casper_client::{
    get_state_root_hash, query_global_state, types::StoredValue, JsonRpcId, Verbosity,
};
use casper_event_standard::casper_types::{bytesrepr::FromBytes, ContractHash, Key};
use casper_event_standard::Schemas;
use casper_event_toolkit::fetcher::Fetcher;
use casper_event_toolkit::rpc::compat::key_to_client_types;
use contract_utils::constants::KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER;
use kairos_circuit_logic::transactions::{KairosTransaction, L1Deposit};

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

pub async fn on_deploy_notification(
    event_fetcher: &Fetcher,
    event_schemas: &Schemas,
    casper_node_rpc_url: &Url,
    contract_hash: &ContractHash,
    batch_state_manager: &BatchStateManager,
) {
    let last_unprocessed_deposit_index = event_fetcher
        .fetch_events_count()
        .await
        .expect("Failed to fetch the last unprocessed deposit index");
    // FIXME in demo-contract to u32
    let last_processed_deposit_index: u32 =
        get_last_deposit_counter(casper_node_rpc_url, contract_hash)
            .await
            .expect("Failed to fetch the index for the last processed deposit");

    for deposit_index in (last_processed_deposit_index + 1)..=last_unprocessed_deposit_index {
        let untyped_event = event_fetcher
            .fetch_event(deposit_index, event_schemas)
            .await
            .expect("Failed to fetch the untyped deposit event");
        match untyped_event.name.as_str() {
            "Deposit" => {
                let data = untyped_event
                    .to_ces_bytes()
                    .expect("Failed convert the untyped deposit event into bytes");
                let (deposit, _) = contract_utils::Deposit::from_bytes(&data)
                    .expect("Failed to parse deposit event from bytes");
                batch_state_manager
                    .enqueue_transaction(KairosTransaction::Deposit(L1Deposit {
                        recipient: "cafebabe".into(),
                        amount: deposit.amount,
                    }))
                    .await
                    .expect("Failed to enque deposit");
            }
            other => {
                tracing::error!("Unknown event type: {}", other)
            }
        }
    }
}
