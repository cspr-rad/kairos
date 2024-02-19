use casper_client::{get_dictionary_item, get_state_root_hash, JsonRpcId, Verbosity};
use casper_types::URef;
use contract_types::Deposit;
use serde_json;
extern crate alloc;
use alloc::{string::ToString, vec::Vec};

/*
    GET_DEPOSIT {
        node_address: String,
        rpc_port: String,
        dict_uref: String,
        key: String
    }
*/

pub async fn get(node_address: &str, rpc_port: String, dict_uref: &str, key: String) -> Deposit {
    let srh = get_state_root_hash(
        JsonRpcId::String(rpc_port.clone()),
        node_address,
        Verbosity::Low,
        None,
    )
    .await
    .unwrap()
    .result
    .state_root_hash
    .unwrap();

    let item = get_dictionary_item(
        JsonRpcId::String(rpc_port),
        node_address,
        Verbosity::Low,
        srh,
        casper_client::rpcs::DictionaryItemIdentifier::URef {
            seed_uref: URef::from_formatted_str(dict_uref).unwrap(),
            dictionary_item_key: key,
        },
    )
    .await
    .unwrap()
    .result
    .stored_value;
    let item_bytes: Vec<u8> = item.into_cl_value().unwrap().into_t().unwrap();
    let item_deserialized: Deposit = serde_json::from_slice(&item_bytes).unwrap();
    item_deserialized
}
