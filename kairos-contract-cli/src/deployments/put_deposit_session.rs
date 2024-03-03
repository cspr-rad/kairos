use crate::deployments::deploy_handler::LayerOneDeployHandler;
use casper_client::rpcs::results::PutDeployResult;
use casper_client::JsonRpcId;
use casper_types::{
    addressable_entity::AddressableEntityHash, runtime_args, ExecutableDeployItem, SecretKey, U512,
};
use std::fs;
/*

    node_address: String,
    rpc_port: String,
    secret_key_path: String,
    chain_name: String,
    wasm_path: String,
    contract_addr: String,
    amount: U512

*/
pub async fn put(
    node_address: String,
    rpc_port: String,
    secret_key_path: String,
    chain_name: &str,
    wasm_path: &str,
    contract_addr: &str,
    amount: U512,
) -> PutDeployResult {
    let wasm_bytes = fs::read(wasm_path).unwrap();
    let raw_deploy = ExecutableDeployItem::ModuleBytes {
        module_bytes: wasm_bytes.into(),
        // transfer 10 tokens to contract
        args: runtime_args! {
            "amount" => amount,
            "deposit_contract" => AddressableEntityHash::from_formatted_str(contract_addr).unwrap()
        },
    };

    let secret_key_bytes = fs::read(secret_key_path).unwrap();
    let secret_key: SecretKey = SecretKey::from_pem(secret_key_bytes.clone()).unwrap();
    let layer_one_deploy_handler = LayerOneDeployHandler {
        node_address,
        rpc_port: JsonRpcId::String(rpc_port),
        secret_key,
    };
    let mut deploy = layer_one_deploy_handler.build_deploy(
        chain_name,
        raw_deploy,
        &SecretKey::from_pem(secret_key_bytes.clone()).unwrap(),
        100_000_000_000u64,
    );
    deploy.sign(&SecretKey::from_pem(secret_key_bytes).unwrap());
    layer_one_deploy_handler.put_deploy(deploy).await
}
