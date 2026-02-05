//! # Handlers Module
//!
//! This module contains the business logic for the API endpoints.
//! Handlers are responsible for processing requests and returning
//! appropriate HTTP responses.

use axum::{response::IntoResponse, Json};
use serde_json::json;

pub mod removebg;

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