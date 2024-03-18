use casper_client::{get_state_root_hash, query_global_state, get_dictionary_item, JsonRpcId, Verbosity};
use casper_types::{Key, URef, U512};
use kairos_risc0_types::Deposit;

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

pub async fn query_dictionary(
    node_address: &str,
    rpc_port: String,
    dict_uref: URef,
    key: String
) -> Deposit{
    let srh = query_state_root_hash(node_address, rpc_port).await;
    //let
}