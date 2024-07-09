use axum::{extract::State, Json};
use axum_extra::routing::TypedPath;
use tracing::*;

use crate::{state::ServerState, AppErr, PublicKey};

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/nonce")]
pub struct GetNoncePath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn get_nonce_handler(
    _: GetNoncePath,
    state: State<ServerState>,
    Json(body): Json<PublicKey>,
) -> Result<Json<u64>, AppErr> {
    let nonce = state.batch_state_manager.get_nonce_for(body).await?;
    Ok(Json(nonce))
}
