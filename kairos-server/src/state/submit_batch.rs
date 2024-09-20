use std::time::Instant;

use anyhow::anyhow;
use backoff::{future::retry, ExponentialBackoff};
use casper_client::{Error, JsonRpcId};
use casper_types::{
    bytesrepr::Bytes,
    contracts::ContractHash,
    execution::{execution_result_v1::ExecutionResultV1, ExecutionResult},
    runtime_args, DeployBuilder, ExecutableDeployItem, SecretKey, TimeDiff, Timestamp,
};
use rand::random;
use reqwest::Url;
use risc0_zkvm::Receipt;

use crate::routes::get_chain_name::get_chain_name_from_rpc;

pub const MAX_GAS_FEE_PAYMENT_AMOUNT: u64 = 10_000_000_000_000;
// TODO: retry request on failure, improve error handling
pub async fn submit_proof_to_contract(
    signer: &SecretKey,
    contract_hash: ContractHash,
    casper_rpc: Url,
    receipt: &Receipt,
) {
    let proof_serialized = Bytes::from(serde_json::to_vec(receipt).expect("could not serialize"));

    tracing::info!("Submitting proof to contract: {:?}", contract_hash);
    let submit_batch = ExecutableDeployItem::StoredContractByHash {
        hash: contract_hash.into(),
        entry_point: "submit_batch".into(),
        args: runtime_args! {
            "risc0_receipt" => proof_serialized,
        },
    };

    let chain_name = get_chain_name_from_rpc(&casper_rpc)
        .await
        .expect("RPC request failed");
    let deploy = DeployBuilder::new(chain_name, submit_batch)
        .with_secret_key(signer)
        .with_standard_payment(MAX_GAS_FEE_PAYMENT_AMOUNT)
        .with_timestamp(Timestamp::now())
        .with_ttl(TimeDiff::from_millis(60_000))
        .build()
        .expect("could not build deploy");

    let deploy_hash = *deploy.hash();

    let r = casper_client::put_deploy(
        casper_client::JsonRpcId::Number(random()),
        casper_rpc.as_str(),
        casper_client::Verbosity::Low,
        deploy,
    )
    .await
    .expect("could not put deploy");

    let start = Instant::now();
    let timed_out = start.elapsed().as_secs() > 60;

    retry(ExponentialBackoff::default(), || async {
        let response = casper_client::get_deploy(
            JsonRpcId::Number(1),
            casper_rpc.as_str(),
            casper_client::Verbosity::Low,
            deploy_hash,
            false,
        )
        .await
        .map_err(|err| {
            let elapsed = start.elapsed().as_secs();
            tracing::info!("Running for {elapsed}s, Error: {err:?}");
            err
        })
        .map_err(|err| match &err {
            e if timed_out => backoff::Error::permanent(anyhow!("Timeout on error: {e:?}")),
            Error::ResponseIsHttpError { .. } | Error::FailedToGetResponse { .. } => {
                backoff::Error::transient(anyhow!(err))
            }
            _ => backoff::Error::permanent(anyhow!(err)),
        })
        .expect("could not get deploy");

        match response.result.execution_info {
            Some(execution_info) => match execution_info.execution_result {
                Some(execution_result) => match &execution_result {
                    ExecutionResult::V1(execution_result_v1) => match execution_result_v1 {
                        ExecutionResultV1::Failure { error_message, .. } => {
                            Err(backoff::Error::permanent(anyhow!(error_message.clone())))
                        }
                        ExecutionResultV1::Success { .. } => Ok(()),
                    },
                    ExecutionResult::V2(execution_result_v2) => {
                        match &execution_result_v2.error_message {
                            None => Ok(()),
                            Some(error_message) => {
                                Err(backoff::Error::permanent(anyhow!(error_message.clone())))
                            }
                        }
                    }
                },
                None if timed_out => Err(backoff::Error::permanent(anyhow!(
                    "Timeout on error: No execution result"
                ))),
                None => Err(backoff::Error::transient(anyhow!(
                    "No execution results there yet"
                ))),
            },
            None if timed_out => Err(backoff::Error::permanent(anyhow!(
                "Timeout on error: No execution info"
            ))),
            None => Err(backoff::Error::transient(anyhow!(
                "No execution results there yet"
            ))),
        }
    })
    .await
    .expect("could not get deploy or deploy failed");

    tracing::info!("Deploy successful: {:?}", r);
}
