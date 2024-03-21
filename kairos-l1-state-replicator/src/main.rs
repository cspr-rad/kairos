#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    // Fetch some deploy:
    // - https://cspr.live/deploy/394ff2bbec812397d6d55359d55f9e734266f3e61dc92a14ac0f4741a2bbf000
    let rpc_id: casper_client::JsonRpcId = 1.into();
    let node_address: &str = "https://mainnet.casper-node.xyz";
    let verbosity = casper_client::Verbosity::Low;
    let deploy_hash = casper_client::types::DeployHash::new([57, 79, 242, 187, 236, 129, 35, 151, 214, 213, 83, 89, 213, 95, 158, 115, 66, 102, 243, 230, 29, 201, 42, 20, 172, 15, 71, 65, 162, 187, 240, 0].into());
    let finalized_approvals: bool = false;
    let deploy_result = casper_client::get_deploy(
        rpc_id,
        node_address,
        verbosity,
        deploy_hash,
        finalized_approvals,
    ).await?.result;
    println!("Deploy: {:?}", deploy_result);

    // Contract correlated with deploy:
    // - https://cspr.live/contract/d16aa9c6a7dc03d0f422eafeb244dcc782a3b7372bc2b2245c4e04f9c93f3e8f
    // TODO: See if this can be obtained automatically.
    // NOTE: ces-go-parser observes array of contract hashes.
    let contract_hash = casper_types::ContractWasmHash::new([209, 106, 169, 198, 167, 220, 3, 208, 244, 34, 234, 254, 178, 68, 220, 199, 130, 163, 183, 55, 43, 194, 178, 36, 92, 78, 4, 249, 201, 63, 62, 143]);

    // Fetch latest state root hash.
    let rpc_id: casper_client::JsonRpcId = 2.into();
    let node_address: &str = "https://mainnet.casper-node.xyz";
    let verbosity = casper_client::Verbosity::Low;
    let state_root_hash_result = casper_client::get_state_root_hash(rpc_id, node_address, verbosity, None).await?.result;
    let state_root_hash = state_root_hash_result.state_root_hash.unwrap(); // TODO: Handle no value.
    println!("State root hash: {:?}", state_root_hash);

    // Fetch contract details.
    let rpc_id: casper_client::JsonRpcId = 2.into();
    let node_address: &str = "https://mainnet.casper-node.xyz";
    let verbosity = casper_client::Verbosity::Low;
    let global_state_identifier = casper_client::rpcs::GlobalStateIdentifier::StateRootHash(state_root_hash);
    let key = casper_types::Key::Hash(contract_hash.value());
    let path = vec![];
    let state_result = casper_client::query_global_state(rpc_id, node_address, verbosity, global_state_identifier, key, path).await?.result;
    let contract = match state_result.stored_value {
        casper_client::types::StoredValue::Contract(v) => {
            Ok(v)
        },
        _ => Err("Expected contract.")
    }?;
    println!("Contract: {:?}", contract);

    Ok(())
}
