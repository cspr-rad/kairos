use axum::{
    routing::{get, post},
    Json, Router,
    extract:: State,
};

use crate::AppState;

pub fn delta_tree_routes() -> Router<AppState> {
    Router::new()
        .route("/test", get(|_: State<AppState>| async {}))
}