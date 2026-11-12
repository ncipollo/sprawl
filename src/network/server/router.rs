//! The route table.

use super::handlers::{list_handler, refresh_handler, section_handler};
use super::state::AppState;
use axum::Router;
use axum::routing::{get, post};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/sections", get(list_handler))
        .route("/sections/{id}", get(section_handler))
        .route("/sections/{id}/refresh", post(refresh_handler))
        .with_state(state)
}
