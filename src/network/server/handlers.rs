//! Thin request handlers: each hands off to the section service on a
//! blocking thread (scripts run boa and shell commands) and maps the
//! outcome to a status code.

use super::state::AppState;
use crate::feature::script::schema::SectionConfig;
use crate::feature::server::error::ServiceError;
use crate::feature::server::service::{SectionService, SectionSummary};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tokio::task;

/// `GET /sections`
pub async fn list_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<SectionSummary>>, ApiError> {
    run_blocking(state.service, |service| service.list())
        .await
        .map(Json)
}

/// `GET /sections/{id}`
pub async fn section_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SectionConfig>, ApiError> {
    run_blocking(state.service, move |service| service.layout(&id))
        .await
        .map(Json)
}

/// `POST /sections/{id}/refresh`
pub async fn refresh_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SectionConfig>, ApiError> {
    run_blocking(state.service, move |service| service.refresh(&id))
        .await
        .map(Json)
}

async fn run_blocking<T, F>(service: SectionService, work: F) -> Result<T, ApiError>
where
    T: Send + 'static,
    F: FnOnce(SectionService) -> Result<T, ServiceError> + Send + 'static,
{
    task::spawn_blocking(move || work(service))
        .await
        .map_err(|error| ApiError::Internal(error.to_string()))?
        .map_err(ApiError::Service)
}

/// A failed request, rendered as `{"error": "..."}`.
pub enum ApiError {
    Service(ServiceError),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Service(error) => (status_of(&error), error.to_string()),
            ApiError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

fn status_of(error: &ServiceError) -> StatusCode {
    match error {
        ServiceError::UnknownSection(_) => StatusCode::NOT_FOUND,
        ServiceError::NotRendered(_) | ServiceError::Render(_) => StatusCode::BAD_GATEWAY,
        ServiceError::Io(_) | ServiceError::Corrupt(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
