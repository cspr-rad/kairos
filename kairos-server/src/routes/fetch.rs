use axum::{extract::State, Json};
use axum_extra::routing::TypedPath;
use chrono::NaiveDateTime;
use kairos_data::transaction::*;
use serde::Deserialize;
use tracing::instrument;

use crate::{state::ServerState, AppErr};

#[derive(Deserialize, Debug)]
pub struct QueryTransactionsPayload {
    sender: Option<String>,
    min_timestamp: Option<String>,
    max_timestamp: Option<String>,
    min_amount: Option<Transaction>,
    max_amount: Option<i64>,
    recipient: Option<String>,
}

#[derive(TypedPath)]
#[typed_path("/api/v1/transactions")]
pub struct QueryTransactionsPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn query_transactions_handler(
    _: QueryTransactionsPath,
    State(state): State<ServerState>,
    Json(payload): Json<QueryTransactionsPayload>,
) -> Result<Json<Vec<Transaction>>, AppErr> {
    let filter = TransactionFilter {
        sender: payload.sender,
        min_timestamp: payload
            .min_timestamp
            .and_then(|ts| NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S").ok()),
        max_timestamp: payload
            .max_timestamp
            .and_then(|ts| NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S").ok()),
        min_amount: payload.min_amount,
        max_amount: payload.max_amount,
        recipient: payload.recipient,
    };

    let transactions = get(&state.pool, filter).await?;

    Ok(Json(transactions))
}
