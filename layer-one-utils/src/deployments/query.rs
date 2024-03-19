use crate::constants::{
    CCTL_DEFAULT_NODE_ADDRESS, CCTL_DEFAULT_NODE_RPC_PORT, FORMATTED_COUNTER_UREF,
    FORMATTED_DEPOSIT_EVENT_DICT_UREF, FORMATTED_PROOF_DICT_UREF,
};
use casper_client::{
    get_dictionary_item, get_state_root_hash, query_global_state, types::StoredValue, JsonRpcId,
    Verbosity,
};
use casper_hashing::Digest;
use casper_types::{
    bytesrepr::{Bytes, ToBytes},
    CLValue, Key, URef,
};
use kairos_risc0_types::{Deposit, KairosDeltaTree, RiscZeroProof};
use serde_json;

pub async fn query_state_root_hash(node_address: &str, rpc_port: &str) -> Digest {
    let srh = get_state_root_hash(
        JsonRpcId::String(rpc_port.to_owned()),
        node_address,
        Verbosity::Low,
        None,
    )
    .await
    .unwrap()
    .result
    .state_root_hash
    .unwrap();
    srh
}

pub async fn query_counter(node_address: &str, rpc_port: &str, counter_uref: &str) -> u64 {
    let srh: Digest = query_state_root_hash(node_address, rpc_port).await;
    let stored_value: StoredValue = query_global_state(
        JsonRpcId::String(rpc_port.to_owned()),
        node_address,
        Verbosity::Low,
        Some(casper_client::rpcs::GlobalStateIdentifier::StateRootHash(
            srh,
        )),
        casper_types::Key::URef(URef::from_formatted_str(counter_uref).unwrap()),
        Vec::new(),
    )
    .await
    .unwrap()
    .result
    .stored_value;
    let value: u64 = match stored_value {
        StoredValue::CLValue(cl_value) => return cl_value.into_t().unwrap(),
        _ => return 999999999999u64,
    };
}

pub async fn query_deposit(
    node_address: &str,
    rpc_port: &str,
    dict_uref: &str,
    key: String,
) -> Deposit {
    let srh = query_state_root_hash(node_address, &rpc_port).await;
    let item = get_dictionary_item(
        JsonRpcId::String(rpc_port.to_owned()),
        node_address,
        Verbosity::Low,
        srh,
        casper_client::rpcs::DictionaryItemIdentifier::URef {
            seed_uref: URef::from_formatted_str(dict_uref).unwrap(),
            dictionary_item_key: key,
        },
    )
    .await
    .unwrap();
    let stored_value: StoredValue = item.result.stored_value;
    let deposit_bytes: Bytes = match stored_value {
        StoredValue::CLValue(cl_value) => cl_value.into_t().unwrap(),
        _ => Bytes::from(vec![]),
    };
    let deposit_deserialized: Deposit = serde_json::from_slice(deposit_bytes.as_slice()).unwrap();
    deposit_deserialized
}

pub async fn query_proof(
    node_address: &str,
    rpc_port: &str,
    dict_uref: &str,
    key: String,
) -> RiscZeroProof {
    let srh = query_state_root_hash(node_address, &rpc_port).await;
    let item = get_dictionary_item(
        JsonRpcId::String(rpc_port.to_owned()),
        node_address,
        Verbosity::Low,
        srh,
        casper_client::rpcs::DictionaryItemIdentifier::URef {
            seed_uref: URef::from_formatted_str(dict_uref).unwrap(),
            dictionary_item_key: key,
        },
    )
    .await
    .unwrap();
    let stored_value: StoredValue = item.result.stored_value;
    let proof_bytes: Bytes = match stored_value {
        StoredValue::CLValue(cl_value) => cl_value.into_t().unwrap(),
        _ => Bytes::from(vec![]),
    };
    let proof_deserialized: RiscZeroProof = bincode::deserialize(proof_bytes.as_slice()).unwrap();
    proof_deserialized
}

#[tokio::test]
async fn test_query_counter() {
    let response = query_counter(
        CCTL_DEFAULT_NODE_ADDRESS,
        CCTL_DEFAULT_NODE_RPC_PORT,
        FORMATTED_COUNTER_UREF,
    )
    .await;
    println!("Response: {:?}", &response);
}

#[tokio::test]
async fn test_query_deposit() {
    // query the deposit at index 0
    let response = query_deposit(
        CCTL_DEFAULT_NODE_ADDRESS,
        CCTL_DEFAULT_NODE_RPC_PORT,
        FORMATTED_DEPOSIT_EVENT_DICT_UREF,
        "0".to_string(),
    )
    .await;
    println!("Response: {:?}", &response);
}

#[tokio::test]
async fn test_query_proof() {
    let response = query_proof(
        CCTL_DEFAULT_NODE_ADDRESS,
        CCTL_DEFAULT_NODE_RPC_PORT,
        FORMATTED_PROOF_DICT_UREF,
        "0".to_string(),
    )
    .await;
    println!("Response: {:?}", &response);
}
