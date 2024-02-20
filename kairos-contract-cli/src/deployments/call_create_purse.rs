use crate::deployments::deploy_handler::LayerOneDeployHandler;
use casper_client::{rpcs::results::PutDeployResult, JsonRpcId};
use casper_types::{
    addressable_entity::AddressableEntityHash, crypto::SecretKey, runtime_args,
    ExecutableDeployItem,
};
use std::fs;

/*
    node_address: String,
    rpc_port: String,
    secret_key_path: String,
    chain_name: String,
    contract_addr: String
*/
pub async fn call(
    node_address: String,
    rpc_port: String,
    secret_key_path: String,
    chain_name: &str,
    contract_addr: &str,
) -> PutDeployResult {
    let raw_deploy = ExecutableDeployItem::StoredContractByHash {
        hash: AddressableEntityHash::from_formatted_str(contract_addr).unwrap(),
        entry_point: "create_purse".to_string(),
        args: runtime_args! {},
    };

    let secret_key_bytes = fs::read(secret_key_path).unwrap();
    let secret_key: SecretKey = SecretKey::from_pem(secret_key_bytes.clone()).unwrap();
    let layer_one_deploy_handler = LayerOneDeployHandler {
        node_address: node_address,
        rpc_port: JsonRpcId::String(rpc_port),
        secret_key,
    };
    let mut deploy = layer_one_deploy_handler.build_deploy(
        chain_name,
        raw_deploy,
        &SecretKey::from_pem(secret_key_bytes.clone()).unwrap(),
        10_000_000_0000u64,
    );
    deploy.sign(&SecretKey::from_pem(secret_key_bytes).unwrap());
    let deploy_result = layer_one_deploy_handler.put_deploy(deploy).await;
    deploy_result
}
