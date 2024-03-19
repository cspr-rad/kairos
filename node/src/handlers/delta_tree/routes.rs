use axum::{
    routing::{get, post},
    Json, Router,
    extract:: State,
};

use crate::AppState;

pub fn delta_tree_routes() -> Router<AppState> {
    Router::new()
        .route("/transfer", get(|_: State<AppState>| async {}))
        .route("/submit_batch", post(|_: State<AppState>| async {}))
}