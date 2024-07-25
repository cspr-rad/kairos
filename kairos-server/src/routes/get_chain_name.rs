use axum::{extract::State, Json};
use axum_extra::routing::TypedPath;
use tracing::*;

use crate::{state::ServerState, AppErr};

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/chain_name")]
pub struct GetChainNamePath;

#[instrument(level = "trace", skip(_state), ret)]
pub async fn get_chain_name_handler(
    _: GetChainNamePath,
    _state: State<ServerState>,
) -> Result<Json<String>, AppErr> {
    let chain_name = env!("CASPER_CHAIN_NAME"); // NOTE: This should be obtained from RPC, rather than from hardcoded value.
    Ok(Json(String::from(chain_name)))
}

pub async fn get_chain_name_from_rpc(rpc_url: &str) -> Result<String, ()> {
    let request_id = casper_client::JsonRpcId::Number(1);
    let verbosity = casper_client::Verbosity::Low;
    let response = casper_client::get_node_status(request_id, rpc_url, verbosity).await.expect("RPC request failed");
    let chain_name = response.result.chainspec_name;

    Ok(chain_name)
}
