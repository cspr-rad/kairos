use std::any::Any;

use casper_client::{get_state_root_hash, query_global_state, get_dictionary_item, JsonRpcId, Verbosity};
use casper_types::{Key, URef, U512};
use crate::constants::{CCTL_DEFAULT_NODE_ADDRESS, CCTL_DEFAULT_NODE_RPC_PORT, FORMATTED_COUNTER_UREF, FORMATTED_DEPOSIT_EVENT_DICT_UREF};
use kairos_risc0_types::Deposit;
use serde_json;

pub async fn query_state_root_hash(
    node_address: &str,
    rpc_port: String
) -> Digest{
    let srh = get_state_root_hash(
        JsonRpcId::String(rpc_port), 
        node_address, 
        Verbosity::Low, 
        None
    ).await.unwrap().result.state_root_hash.unwrap();
    srh
}

pub async fn query_counter(
    node_address: &str,
    rpc_port: String,
    counter_uref: URef
) -> U512{
    let srh = query_state_root_hash(node_address, rpc_port).await;
    let value: U512 = query_global_state(
        JsonRpcId::String(rpc_port), 
        node_address, 
        Verbosity::Low, 
        casper_client::rpcs::GlobalStateIdentifier::StateRootHash(srh), 
        casper_types::Key::URef(counter_uref), 
        Vec::new()
    ).await.unwrap().result.into().unwrap();
    value
}

pub async fn query_deposit(
    node_address: &str,
    rpc_port: String,
    dict_uref: URef,
    key: String
) -> Deposit{
    let srh = query_state_root_hash(node_address, rpc_port).await;
    let item = get_dictionary_item(
        JsonRpcId::String(rpc_port), 
        node_address, 
        Verbosity::Low, 
        srh, 
        casper_client::rpcs::DictionaryItemIdentifier::URef { 
            seed_uref: URef::from_formatted_str(dict_uref).unwrap(), 
            dictionary_item_key: key 
        }
    ).await.unwrap();
    let deposit_bytes: Vec<u8> = item.result.stored_value;
    let deposit_deserialized: Deposit = serde_json::from_slice(&deposit_bytes).unwrap();
    deposit_deserialized
}

#[tokio::test]
async fn test_query_counter(){
    let response = query_counter(CCTL_DEFAULT_NODE_ADDRESS, CCTL_DEFAULT_NODE_RPC_PORT.to_string(), FORMATTED_COUNTER_UREF).await;
    println!("Response: {:?}", &response);
}

#[tokio::test]
async fn test_query_deposit(){
    // query the deposit at index 0
    let response = query_deposit(CCTL_DEFAULT_NODE_ADDRESS, CCTL_DEFAULT_NODE_RPC_PORT.to_string(), FORMATTED_DEPOSIT_EVENT_DICT_UREF, "0".to_string()).await;
    println!("Response: {:?}", &response);
}