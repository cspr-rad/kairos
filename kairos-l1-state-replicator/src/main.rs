use std::collections::BTreeMap;

use casper_event_standard::CLType2;
use casper_types::bytesrepr::{FromBytes, ToBytes};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch some deploy:
    // - https://cspr.live/deploy/5fc34e15776a08bd059355acd57937b56ddc48ad0c2f55bd8d0b376170c8a412
    let rpc_id: casper_client::JsonRpcId = 1.into();
    let node_address: &str = "https://mainnet.casper-node.xyz";
    let verbosity = casper_client::Verbosity::Low;
    let deploy_hash = casper_client::types::DeployHash::new(
        [
            95, 195, 78, 21, 119, 106, 8, 189, 5, 147, 85, 172, 213, 121, 55, 181, 109, 220, 72,
            173, 12, 47, 85, 189, 141, 11, 55, 97, 112, 200, 164, 18,
        ]
        .into(),
    );
    let finalized_approvals: bool = false;
    let deploy_result = casper_client::get_deploy(
        rpc_id,
        node_address,
        verbosity,
        deploy_hash,
        finalized_approvals,
    )
    .await?
    .result;

    //println!("Deploy: {:?}", deploy_result);

    // Contract correlated with deploy:
    // - https://cspr.live/contract/fe03021407879ce6fc5e035b70ff6a90941afdbea325a9164c7a497827efa7ff
    // TODO: See if this can be obtained automatically.
    // NOTE: ces-go-parser observes array of contract hashes.
    let contract_hash = casper_types::ContractWasmHash::new([
        254, 3, 2, 20, 7, 135, 156, 230, 252, 94, 3, 91, 112, 255, 106, 144, 148, 26, 253, 190,
        163, 37, 169, 22, 76, 122, 73, 120, 39, 239, 167, 255,
    ]);

    // Fetch latest state root hash.
    let rpc_id: casper_client::JsonRpcId = 2.into();
    let node_address: &str = "https://mainnet.casper-node.xyz";
    let verbosity = casper_client::Verbosity::Low;
    let state_root_hash_result =
        casper_client::get_state_root_hash(rpc_id, node_address, verbosity, None)
            .await?
            .result;
    let state_root_hash = state_root_hash_result.state_root_hash.unwrap(); // TODO: Handle no value.

    //println!("State root hash: {:?}", state_root_hash);

    // Fetch contract details.
    let rpc_id: casper_client::JsonRpcId = 2.into();
    let node_address: &str = "https://mainnet.casper-node.xyz"; // TODO FIX TESTNTET
    let verbosity = casper_client::Verbosity::Low;
    let global_state_identifier =
        casper_client::rpcs::GlobalStateIdentifier::StateRootHash(state_root_hash);
    let key = casper_types::Key::Hash(contract_hash.value());
    let path = vec![];
    let state_result = casper_client::query_global_state(
        rpc_id,
        node_address,
        verbosity,
        global_state_identifier,
        key,
        path,
    )
    .await?
    .result;
    let contract = match state_result.stored_value {
        casper_client::types::StoredValue::Contract(v) => Ok(v),
        _ => Err("Expected contract."),
    }?;

    //println!("Contract: {:?}", contract);

    // Load contract metadata without schema.
    let mut events_schema_uref: Option<String> = None;
    let mut events_uref: Option<String> = None;
    for named_key in contract.named_keys() {
        if named_key.name() == "__events_schema" {
            events_schema_uref = Some(named_key.key().unwrap().to_formatted_string());
        }
        if named_key.name() == "__events" {
            events_uref = Some(named_key.key().unwrap().to_formatted_string());
        }
    }
    let (events_schema_uref, events_uref) = match (events_schema_uref, events_uref) {
        (Some(events_schema_uref), Some(events_uref)) => Ok((events_schema_uref, events_uref)),
        _ => Err("Expected named keys."),
    }?;

    //println!("Events schema uref: {:?}", events_schema_uref);
    //println!("Events uref: {:?}", events_uref);

    // Load contract event schemas.
    let rpc_id: casper_client::JsonRpcId = 3.into();
    let node_address: &str = "https://mainnet.casper-node.xyz";
    let verbosity = casper_client::Verbosity::Low;
    let global_state_identifier =
        casper_client::rpcs::GlobalStateIdentifier::StateRootHash(state_root_hash);
    let key = casper_types::Key::URef(
        casper_types::URef::from_formatted_str(&events_schema_uref).unwrap(),
    );
    let path = vec![];
    let state_result = casper_client::query_global_state(
        rpc_id,
        node_address,
        verbosity,
        global_state_identifier,
        key,
        path,
    )
    .await?
    .result;
    let schema_value = match state_result.stored_value {
        casper_client::types::StoredValue::CLValue(v) => Ok(v),
        _ => Err("Expected CLValue."),
    }?;

    //println!("Events schema: {:?}", schema_value);

    // We cannot parse CLValue based on the CLType from the Argument raw data, as it may contain an Any type
    // which we do not know how to parse. Therefore, we should parse the raw bytes, ignore the clType field,
    // and provide the hardcoded CLType with the cltype.Dynamic type instead of Any
    let schema_bytes = schema_value
        .to_bytes()
        .map_err(|_e| "Unable to get schema bytes.")?;
    let (parsed, remainder) = casper_types::CLValue::from_bytes(&schema_bytes)
        .map_err(|_e| "Unable to parse schema bytes.")?;
    assert!(remainder.len() == 0);
    let events_schema: BTreeMap<String, Vec<(String, CLType2)>> = parsed.into_t().unwrap();
    
    println!("Events schema parsed: {:?}", events_schema);

    Ok(())
}
