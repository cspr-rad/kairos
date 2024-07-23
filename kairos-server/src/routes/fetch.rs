use axum::{extract::State, Json};
use axum_extra::routing::TypedPath;
use kairos_data::transaction::*;
use tracing::instrument;

use crate::{state::ServerState, AppErr};

#[derive(TypedPath)]
#[typed_path("/api/v1/transactions")]
pub struct QueryTransactionsPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn query_transactions_handler(
    _: QueryTransactionsPath,
    State(state): State<ServerState>,
    Json(filter): Json<TransactionFilter>,
) -> Result<Json<Vec<Transactions>>, AppErr> {
    get(&state.pool, filter).await.map(Json).map_err(Into::into)
}
