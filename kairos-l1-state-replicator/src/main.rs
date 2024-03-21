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

    // Fetch latest state root hash.
    let rpc_id: casper_client::JsonRpcId = 2.into();
    let node_address: &str = "https://mainnet.casper-node.xyz";
    let verbosity = casper_client::Verbosity::Low;
    let state_root_hash_result = casper_client::get_state_root_hash(rpc_id, node_address, verbosity, None).await?.result;
    let state_root_hash = state_root_hash_result.state_root_hash.unwrap(); // TODO: Handle no value.
    println!("State root hash: {:?}", state_root_hash);

    Ok(())
}
