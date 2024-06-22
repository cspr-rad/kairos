use anyhow::anyhow;
use axum::{extract::State, Json};
use axum_extra::routing::TypedPath;
use rand::Rng;
use tracing::*;

use crate::{state::ServerState, AppErr};
use casper_client::{
    put_deploy,
    types::{Deploy, DeployHash},
    JsonRpcId,
};

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/deposit")]
pub struct DepositPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn deposit_handler(
    _: DepositPath,
    state: State<ServerState>,
    Json(body): Json<Deploy>,
) -> Result<Json<DeployHash>, AppErr> {
    let depositor_account = body.header().account();
    let expected_rpc_id = JsonRpcId::Number(rand::thread_rng().gen::<i64>());
    match body
        .approvals()
        .iter()
        .find(|approval| approval.signer() == depositor_account)
    {
        None => return Err(anyhow!("Deploy not signed by depositor").into()),
        Some(_) => {
            let response = put_deploy(
                expected_rpc_id.clone(),
                state.server_config.casper_rpc.as_str(),
                casper_client::Verbosity::High,
                body,
            )
            .await
            .map_err(Into::<AppErr>::into)?;
            if response.id == expected_rpc_id {
                if state.deposit_manager.is_some() {
                    assert!(state
                        .deposit_manager
                        .as_ref()
                        .unwrap()
                        .known_deposit_deploys
                        .write()
                        .await
                        .insert(response.result.deploy_hash));
                }
                Ok(Json(response.result.deploy_hash))
            } else {
                Err(anyhow!("JSON RPC Id missmatch").into())
            }
        }
    }
}
