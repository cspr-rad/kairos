use axum::{extract::State, Json};
use axum_extra::routing::TypedPath;
use tracing::*;

use crate::{state::ServerState, AppErr};
use casper_types::contracts::ContractHash;

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/contract-hash")]
pub struct ContractHashPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn contract_hash_handler(
    _: ContractHashPath,
    state: State<ServerState>,
) -> Result<Json<ContractHash>, AppErr> {
    Ok(Json(state.server_config.kairos_demo_contract_hash))
}
