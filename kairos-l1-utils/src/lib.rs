use casper_client::{
    get_state_root_hash, types::StoredValue, JsonRpcId,
    Verbosity,
};
pub use casper_client::query_global_state;
use casper_hashing::Digest;
use casper_types::{URef, Key, ContractWasmHash};

use casper_client::{
    rpcs::results::PutDeployResult,
    types::{Deploy, DeployBuilder, ExecutableDeployItem, Timestamp},
    SuccessResponse,
};
use casper_types::{crypto::SecretKey, RuntimeArgs};
use std::fs;

pub const DEFAULT_PAYMENT_AMOUNT: u64 = 1_000_000_000_000;

pub async fn install_wasm_bytecode(
    node_address: &str,
    chain_name: &str,
    runtime_args: RuntimeArgs,
    module_bytes: &[u8],
    secret_key_path: &str,
) -> SuccessResponse<PutDeployResult> {
    let session: ExecutableDeployItem =
        ExecutableDeployItem::new_module_bytes(module_bytes.into(), runtime_args);
    let secret_key_bytes: Vec<u8> = fs::read(secret_key_path).unwrap();
    let secret_key: SecretKey = SecretKey::from_pem(secret_key_bytes.clone()).unwrap();

    let deploy: Deploy = DeployBuilder::new(chain_name, session, &secret_key)
        .with_timestamp(Timestamp::now())
        .with_standard_payment(DEFAULT_PAYMENT_AMOUNT)
        .build()
        .unwrap();

    casper_client::put_deploy(
        JsonRpcId::String(0.to_string()),
        node_address,
        casper_client::Verbosity::Low,
        deploy,
    )
    .await
    .unwrap()
}

pub async fn query_state_root_hash(node_address: &str) -> Digest {
    get_state_root_hash(
        JsonRpcId::String(0.to_string()),
        node_address,
        Verbosity::Low,
        None,
    )
    .await
    .unwrap()
    .result
    .state_root_hash
    .unwrap()
}

pub async fn query_contract_counter(node_address: &str, srh: Digest, contract_hash: Key, path: Vec<String>) -> u64{
    let stored_value: StoredValue = query_global_state(
        JsonRpcId::String(0.to_string()),
        node_address,
        Verbosity::Low,
        casper_client::rpcs::GlobalStateIdentifier::StateRootHash(srh),
        contract_hash.into(),
        path,
    )
    .await
    .expect("Failed to query contract for path")
    .result
    .stored_value;

    let value: u64 = match stored_value{
        StoredValue::CLValue(cl_value) => cl_value.into_t().unwrap(),
        _ => panic!("Missing or invalid Value")
    };

    value
}

pub async fn obtain_contract_uref(node_address: &str, srh: Digest, account: Key, contract_identifier: &str, contract_uref: &str) -> URef {
    let stored_value: StoredValue = query_global_state(
        JsonRpcId::String(0.to_string()),
        node_address,
        Verbosity::Low,
        casper_client::rpcs::GlobalStateIdentifier::StateRootHash(srh),
        account,
        vec![contract_identifier.to_owned()],
    )
    .await
    .unwrap()
    .result
    .stored_value;

    let contract: ContractWasmHash = match stored_value{
        StoredValue::Contract(contract) => *contract.contract_wasm_hash(),
        _ => panic!("Missing or invalid Value - contract identifier is incorrect")
    };

    let stored_value: StoredValue = query_global_state(
        JsonRpcId::String(0.to_string()),
        node_address,
        Verbosity::Low,
        casper_client::rpcs::GlobalStateIdentifier::StateRootHash(srh),
        contract.into(),
        vec![contract_uref.into()],
    )
    .await
    .unwrap()
    .result
    .stored_value;

    let value: URef = match stored_value{
        StoredValue::CLValue(cl_value) => cl_value.into_t().unwrap(),
        _ => panic!("Missing or invalid Value - contract uref does not exist")
    };

    value
}

// #[tokio::test]
// async fn install_wasm() {
//     use std::fs::File;
//     use std::io::Read;
//     use casper_types::{RuntimeArgs, runtime_args};
//     let mut wasm_file: File =
//         File::open("/Users/chef/Desktop/demo-contract-optimized.wasm").unwrap();
//     let mut wasm_bytes: Vec<u8> = Vec::new();
//     wasm_file.read_to_end(&mut wasm_bytes).unwrap();
//     let secret_key_path: &str = "/Users/chef/Desktop/secret_key.pem";
//     let runtime_args: RuntimeArgs = runtime_args! {};
//     let result: SuccessResponse<PutDeployResult> = install_wasm_bytecode(
//         "http://127.0.0.1:11101/rpc",
//         "11101",
//         "cspr-dev-cctl",
//         runtime_args,
//         &wasm_bytes,
//         secret_key_path,
//     )
//     .await;
//     println!("Deploy result: {:?}", &result);
// }
