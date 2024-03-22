use axum::{
    routing::{get, post},
    Json, Router,
    extract:: State,
};

use crate::AppState;
use crate::handlers::delta_tree;

pub fn delta_tree_routes() -> Router<AppState> {
    Router::new()
        .route("/transfer", post(delta_tree::transfer::transfer))
        .route("/submit_batch", get(delta_tree::submit_batch::submit_batch))
}