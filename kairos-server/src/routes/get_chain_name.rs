use axum::{extract::State, Json};
use axum_extra::routing::TypedPath;
use reqwest::Url;
use tracing::*;

use crate::{state::ServerState, AppErr};

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/chain_name")]
pub struct GetChainNamePath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn get_chain_name_handler(
    _: GetChainNamePath,
    state: State<ServerState>,
) -> Result<Json<String>, AppErr> {
    // Call RPC to get chain name.
    let rpc_url = &state.server_config.casper_rpc;
    let chain_name = get_chain_name_from_rpc(rpc_url)
        .await
        .expect("RPC request failed");

    Ok(Json(chain_name))
}

pub async fn get_chain_name_from_rpc(rpc_url: &Url) -> Result<String, ()> {
    let request_id = casper_client::JsonRpcId::Number(1);
    let verbosity = casper_client::Verbosity::Low;
    let response = casper_client::get_node_status(request_id, rpc_url.as_str(), verbosity)
        .await
        .expect("RPC request failed");
    let chain_name = response.result.chainspec_name;

    Ok(chain_name)
}
