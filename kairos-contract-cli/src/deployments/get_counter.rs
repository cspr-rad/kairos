use casper_client::{get_state_root_hash, query_global_state, JsonRpcId, Verbosity};
use casper_types::{URef, U512};
/*
    GET_COUNTER {
        node_address: String,
        rpc_port: String,
        counter_uref: String
    }
*/

pub async fn get(node_address: &str, rpc_port: String, counter_uref: URef) -> U512 {
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

    let value = query_global_state(
        JsonRpcId::String(rpc_port),
        node_address,
        Verbosity::Low,
        casper_client::rpcs::GlobalStateIdentifier::StateRootHash(srh),
        casper_types::Key::URef(counter_uref),
        Vec::new(),
    )
    .await
    .unwrap()
    .result
    .stored_value;
    let value: U512 = value.into_cl_value().unwrap().into_t().unwrap();
    value
}
