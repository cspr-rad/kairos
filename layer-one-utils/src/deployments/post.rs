use casper_client::{rpcs::results::PutDeployResult, JsonRpcId, types::ExecutableDeployItem, types::{Deploy, DeployBuilder}};
use casper_types::{ContractHash, crypto::SecretKey, runtime_args, bytesrepr::ToBytes, RuntimeArgs};
use crate::constants::{DEFAULT_PAYMENT_AMOUNT};
use std::fs;

pub async fn submit_delta_tree_batch(
    node_address: &str,
    rpc_port: &str,
    secret_key_path: &str,
    chain_name: &str,
    contract: &str
){
    let session: ExecutableDeployItem = ExecutableDeployItem::StoredContractByHash { 
        hash: ContractHash::from_formatted_str(contract).unwrap(), 
        entry_point: "submit_delta_tree_batch".to_string(), 
        args: runtime_args!{

        }
    };
    let secret_key_bytes: Vec<u8> = fs::read(secret_key_path).unwrap();
    let secret_key: SecretKey = SecretKey::from_pem(secret_key_bytes.clone()).unwrap();
    
    let deploy: Deploy = DeployBuilder::new(
        chain_name,
        session
    ).with_standard_payment(DEFAULT_PAYMENT_AMOUNT).with_secret_key(&secret_key).build().unwrap();

    let result = casper_client::put_deploy(
        JsonRpcId::String(rpc_port.to_string()), 
        node_address, 
        casper_client::Verbosity::Low, 
        deploy
    ).await.unwrap();
    
    println!("Deploy result: {:?}", &result);
}