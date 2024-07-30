use casper_client::types::{DeployBuilder, ExecutableDeployItem, TimeDiff, Timestamp};
use casper_client_types::{bytesrepr::Bytes, runtime_args, ContractHash, RuntimeArgs, SecretKey};
use rand::random;
use reqwest::Url;

use crate::routes::get_chain_name::get_chain_name_from_rpc;

pub const MAX_GAS_FEE_PAYMENT_AMOUNT: u64 = 10_000_000_000_000;

// TODO: retry request on failure, improve error handling
pub async fn submit_proof_to_contract(
    signer: &SecretKey,
    contract_hash: ContractHash,
    casper_rpc: Url,
    proof_serialized: Vec<u8>,
) {
    let submit_batch = ExecutableDeployItem::StoredContractByHash {
        hash: contract_hash,
        entry_point: "submit_batch".into(),
        args: runtime_args! {
            "risc0_receipt" => Bytes::from(proof_serialized),
        },
    };

    let chain_name = get_chain_name_from_rpc(casper_rpc.as_str())
        .await
        .expect("RPC request failed");
    let deploy = DeployBuilder::new(chain_name, submit_batch, signer)
        .with_standard_payment(MAX_GAS_FEE_PAYMENT_AMOUNT)
        .with_timestamp(Timestamp::now())
        .with_ttl(TimeDiff::from_millis(60_000))
        .build()
        .expect("could not build deploy");

    let r = casper_client::put_deploy(
        casper_client::JsonRpcId::Number(random()),
        casper_rpc.as_str(),
        casper_client::Verbosity::Low,
        deploy,
    )
    .await
    .expect("could not put deploy");

    tracing::info!("Deploy successful: {:?}", r);
}
