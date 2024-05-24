use crate::state::{
    transactions::{Deposit, Signed, Transaction},
    ServerState,
};
use rand::Rng;
use reqwest::Url;

use casper_client::{query_global_state, types::StoredValue, JsonRpcId, Verbosity};
use casper_deploy_notifier::types::Notification;
use casper_event_standard::casper_types::{bytesrepr::FromBytes, CLTyped, HashAddr, Key};
use casper_event_standard::Schemas;
use casper_event_toolkit::fetcher::Fetcher;
use contract_utils;

async fn query_deposit_contract_value<T: CLTyped + FromBytes>(
    casper_node_rpc_url: &Url,
    contract_hash: &HashAddr,
    key: &str,
) -> T {
    let expected_rpc_id = JsonRpcId::Number(rand::thread_rng().gen::<i64>());
    query_global_state(
        expected_rpc_id.clone(),
        casper_node_rpc_url.as_str(),
        Verbosity::High,
        None, // fetches recent blocks state root hash
        Key::Hash(*contract_hash),
        vec![key.to_string()],
    )
    .await
    .map(|response| {
        if response.id == expected_rpc_id {
            match response.result.stored_value {
                StoredValue::CLValue(value) => value
                    .into_t()
                    .expect("Failed to convert from CLValue to desired type"),
                _ => panic!("Unexpected result type, type is not a CLValue"),
            }
        } else {
            panic!("JSON RPC Id missmatch");
        }
    })
    .expect("Failed to query global state")
}

pub async fn on_deploy_notification(
    event_fetcher: &Fetcher,
    event_schemas: &Schemas,
    state: ServerState,
    notification: &Notification,
) {
    let demo_contract_hash = HashAddr::try_from(
        hex::decode(&state.server_config.kairos_demo_contract_hash)
            .expect("Failed to decode the kairos demo contract hash from hex"),
    )
    .expect("Failed to parse the kairos demo contract hash");

    let last_unprocessed_deposit_index = event_fetcher
        .fetch_events_count()
        .await
        .expect("Failed to fetch the last unprocessed deposit index");
    let last_processed_deposit_index: u64 = query_deposit_contract_value(
        &state.server_config.casper_rpc,
        &demo_contract_hash,
        contract_utils::constants::KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER,
    )
    .await;

    for deposit_index in last_processed_deposit_index..(last_unprocessed_deposit_index as u64) {
        let untyped_event = event_fetcher
            .fetch_event(deposit_index, event_schemas)
            .await
            .expect("Failed to fetch the untyped deposit event");
        let nonce = rand::thread_rng().gen::<u64>();
        match untyped_event.name.as_str() {
            "Deposit" => {
                let data = untyped_event
                    .to_ces_bytes()
                    .expect("Failed convert the untyped deposit event into bytes");
                let (deposit, _) = contract_utils::Deposit::from_bytes(&data)
                    .expect("Failed to parse deposit event from bytes");
                state
                    .batch_state_manager
                    .enqueue_transaction(Signed {
                        public_key: notification.public_key.clone().into_bytes(),
                        nonce,
                        transaction: Transaction::Deposit(Deposit {
                            amount: deposit.amount,
                        }),
                    })
                    .await
                    .expect("Failed to enque deposit");
            }
            other => {
                tracing::error!("Unknown event type: {}", other)
            }
        }
    }
}
