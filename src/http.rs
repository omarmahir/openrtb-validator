use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use serde::Serialize;

use crate::{ErrorCode, Severity, ValidationError, validate};

pub fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/validate", post(validate_handler))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

#[derive(Serialize)]
struct ValidateResponse {
    valid: bool,
    errors: Vec<ValidationError>,
}

async fn validate_handler(body: String) -> (StatusCode, Json<ValidateResponse>) {
    let errors = validate(&body);
    let status = status_for(&errors);
    let valid = !errors.iter().any(|e| e.severity == Severity::Error);
    (status, Json(ValidateResponse { valid, errors }))
}

fn status_for(errors: &[ValidationError]) -> StatusCode {
    if errors.iter().any(|e| e.code == ErrorCode::ParseError) {
        StatusCode::BAD_REQUEST
    } else if errors.iter().any(|e| e.severity == Severity::Error) {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    }
}
