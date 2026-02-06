//! # Handlers Module
//!
//! This module contains the business logic for the API endpoints.
//! Handlers are responsible for processing requests and returning
//! appropriate HTTP responses.

use axum::{Json, response::IntoResponse};
use serde_json::json;

pub mod removebg;
pub mod upscaler;

/// Health check handler.
///
/// Returns a JSON response indicating the API status. This can be used
/// by load balancers or monitoring tools to verify the service is running.
///
/// # Returns
///
/// * `200 OK` - Success, returns `{"status": "ok"}`
pub async fn health_check() -> impl IntoResponse {
    tracing::debug!("Health check requested");
    Json(json!({ "status": "ok" }))
}
