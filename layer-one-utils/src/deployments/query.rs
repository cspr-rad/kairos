use casper_client::{get_state_root_hash, query_global_state, get_dictionary_item, JsonRpcId, Verbosity, types::StoredValue};
use casper_types::{bytesrepr::{Bytes, ToBytes}, CLValue, Key, URef};
use casper_hashing::Digest;
use crate::constants::{CCTL_DEFAULT_NODE_ADDRESS, CCTL_DEFAULT_NODE_RPC_PORT, FORMATTED_COUNTER_UREF, FORMATTED_DEPOSIT_EVENT_DICT_UREF};
use kairos_risc0_types::Deposit;
use serde_json;

pub async fn query_state_root_hash(
    node_address: &str,
    rpc_port: &str
) -> Digest{
    let srh = get_state_root_hash(
        JsonRpcId::String(rpc_port.to_owned()), 
        node_address, 
        Verbosity::Low, 
        None
    ).await.unwrap().result.state_root_hash.unwrap();
    srh
}

pub async fn query_counter(
    node_address: &str,
    rpc_port: &str,
    counter_uref: URef
) -> u64{
    let srh: Digest = query_state_root_hash(node_address, rpc_port).await;
    let stored_value: StoredValue = query_global_state(
        JsonRpcId::String(rpc_port.to_owned()), 
        node_address, 
        Verbosity::Low, 
        Some(casper_client::rpcs::GlobalStateIdentifier::StateRootHash(srh)), 
        casper_types::Key::URef(counter_uref), 
        Vec::new()
    ).await.unwrap().result.stored_value;
    let value: u64 = match stored_value{
        StoredValue::CLValue(cl_value) => {
            return cl_value.into_t().unwrap()
        },
        _ => return 666u64
    };
}

pub async fn query_deposit(
    node_address: &str,
    rpc_port: &str,
    dict_uref: URef,
    key: String
) -> Deposit{
    let srh = query_state_root_hash(node_address, &rpc_port).await;
    let item = get_dictionary_item(
        JsonRpcId::String(rpc_port.to_owned()), 
        node_address, 
        Verbosity::Low, 
        srh, 
        casper_client::rpcs::DictionaryItemIdentifier::URef { 
            seed_uref: dict_uref, 
            dictionary_item_key: key 
        }
    ).await.unwrap();
    let stored_value: StoredValue = item.result.stored_value;
    let deposit_bytes: Vec<u8> = match stored_value{
        StoredValue::CLValue(cl_value) => {
            cl_value.into_t().unwrap()
        },
        _ => vec![]
    };
    let deposit_deserialized: Deposit = serde_json::from_slice(&deposit_bytes).unwrap();
    deposit_deserialized
}

#[tokio::test]
async fn test_query_counter(){
    let response = query_counter(CCTL_DEFAULT_NODE_ADDRESS, CCTL_DEFAULT_NODE_RPC_PORT, URef::from_formatted_str(FORMATTED_COUNTER_UREF).unwrap()).await;
    println!("Response: {:?}", &response);
}

#[tokio::test]
async fn test_query_deposit(){
    // query the deposit at index 0
    let response = query_deposit(CCTL_DEFAULT_NODE_ADDRESS, CCTL_DEFAULT_NODE_RPC_PORT, URef::from_formatted_str(FORMATTED_DEPOSIT_EVENT_DICT_UREF).unwrap(), "0".to_string()).await;
    println!("Response: {:?}", &response);
}