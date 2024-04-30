use casper_client::{
    get_state_root_hash, query_global_state, types::StoredValue, JsonRpcId, Verbosity,
};
use casper_hashing::Digest;
use casper_types::{ContractWasmHash, URef};

use casper_client::{
    rpcs::results::PutDeployResult,
    types::{Deploy, DeployBuilder, ExecutableDeployItem, Timestamp},
    SuccessResponse,
};
use casper_types::{crypto::SecretKey, Key, RuntimeArgs};
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
    println!("{}", secret_key_path);
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

pub async fn query_contract_uref_from_installing_account(
    node_address: &str,
    srh: Digest,
    account: Key,
    contract_identifier: &str,
    contract_uref: &str,
) -> URef {
    let stored_value: StoredValue = query_stored_value(
        node_address,
        srh,
        account,
        vec![contract_identifier.to_owned()],
    )
    .await;
    let contract: ContractWasmHash = match stored_value {
        StoredValue::Contract(contract) => *contract.contract_wasm_hash(),
        _ => panic!("Missing or invalid Value"),
    };
    let stored_value: StoredValue = query_stored_value(
        node_address,
        srh,
        contract.into(),
        vec![contract_uref.into()],
    )
    .await;
    let value: URef = match stored_value {
        StoredValue::CLValue(cl_value) => cl_value.into_t().unwrap(),
        _ => panic!("Missing or invalid Value"),
    };

    value
}

pub async fn query_counter(node_address: &str, counter_uref: &str) -> u64 {
    let srh: Digest = query_state_root_hash(node_address).await;
    let stored_value: StoredValue = query_stored_value(
        node_address,
        srh,
        casper_types::Key::URef(URef::from_formatted_str(counter_uref).unwrap()),
        Vec::new(),
    )
    .await;
    let value: u64 = match stored_value {
        StoredValue::CLValue(cl_value) => cl_value.into_t().unwrap(),
        _ => panic!("Missing Value!"),
    };
    value
}

async fn query_stored_value(
    node_address: &str,
    srh: Digest,
    key: Key,
    path: Vec<String>,
) -> StoredValue {
    query_global_state(
        JsonRpcId::String(0.to_string()),
        node_address,
        Verbosity::Low,
        casper_client::rpcs::GlobalStateIdentifier::StateRootHash(srh),
        key,
        path,
    )
    .await
    .unwrap()
    .result
    .stored_value
}

#[cfg_attr(not(feature = "cctl-tests"), ignore)]
#[tokio::test]
async fn install_wasm() {
    use anyhow::anyhow;
    use backoff::{backoff::Constant, future::retry};
    use casper_client::{get_deploy, Error, JsonRpcId, Verbosity};
    use casper_types::{runtime_args, RuntimeArgs};
    use kairos_test_utils::cctl::CCTLNetwork;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    let network = CCTLNetwork::run().await.unwrap();
    let node = network
        .nodes
        .first()
        .expect("Expected at least one node after successful network run");
    let node_address: &str = &format!("http://localhost:{}", node.port.rpc_port);

    let wasm_path = Path::new(env!("PATH_TO_WASM_BINARIES")).join("demo-contract-optimized.wasm");
    let mut wasm_file: File = File::open(wasm_path).unwrap();
    let mut wasm_bytes: Vec<u8> = Vec::new();
    wasm_file.read_to_end(&mut wasm_bytes).unwrap();

    let runtime_args: RuntimeArgs = runtime_args! {};
    let result: SuccessResponse<PutDeployResult> = install_wasm_bytecode(
        node_address,
        "cspr-dev-cctl",
        runtime_args,
        &wasm_bytes,
        network
            .assets_dir
            .join("users/user-1/secret_key.pem")
            .to_str()
            .unwrap(),
    )
    .await;

    // wait for successful processing of deploy
    retry(
        Constant::new(std::time::Duration::from_millis(100)),
        || async {
            get_deploy(
                JsonRpcId::Number(1),
                node_address,
                Verbosity::High,
                result.result.deploy_hash,
                true,
            )
            .await
            .map_err(|err| match &err {
                Error::ResponseIsHttpError { .. } | Error::FailedToGetResponse { .. } => {
                    backoff::Error::transient(anyhow!(err))
                }
                _ => backoff::Error::permanent(anyhow!(err)),
            })
            .map(|success| {
                if success
                    .result
                    .execution_results
                    .iter()
                    .all(|execution_result| match execution_result.result {
                        casper_types::ExecutionResult::Success { .. } => true,
                        casper_types::ExecutionResult::Failure { .. } => false,
                    })
                {
                    Ok(())
                } else {
                    Err(backoff::Error::transient(anyhow!(
                        "Deploy was not processed yet"
                    )))
                }
            })?
        },
    )
    .await
    .unwrap()
}