use casper_client::{
    get_dictionary_item, get_state_root_hash, query_global_state, types::StoredValue, JsonRpcId,
    Verbosity,
};
use casper_hashing::Digest;
use casper_types::{
    bytesrepr::{Bytes, ToBytes},
    CLValue, Key, URef,
};

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
        casper_client::rpcs::GlobalStateIdentifier::StateRootHash(
            srh,
        ),
        casper_types::Key::URef(URef::from_formatted_str(counter_uref).unwrap()),
        Vec::new(),
    )
    .await
    .unwrap()
    .result
    .stored_value;
    let value: u64 = match stored_value {
        StoredValue::CLValue(cl_value) => cl_value.into_t().unwrap(),
        _ => panic!("Missing Value!"),
    };
    value
}

#[tokio::test]
async fn state_root_hash(){
    let srh = query_state_root_hash("http://127.0.0.1:11101/rpc", "11101").await;
    println!("Srh: {:?}", &srh);
}
