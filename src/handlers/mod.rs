//! # Handlers Module
//!
//! This module contains the business logic for the API endpoints.
//! Handlers are responsible for processing requests and returning
//! appropriate HTTP responses.

use crate::AppState;
use axum::{
    Json,
    extract::{Request, State},
    response::IntoResponse,
};
use serde_json::json;

pub mod auth;
pub mod removebg;
pub mod staff;
pub mod upscaler;

use axum::{
    body::Body,
    http::{HeaderMap, StatusCode, header},
    response::Response,
};

/// Health check handler.
///
/// Returns a JSON response indicating the API status. This can be used
/// by load balancers or monitoring tools to verify the service is running.
///
/// # Returns
///
/// * `200 OK` - Success, returns `{"status": "ok"}`
pub async fn health_check(State(_state): State<AppState>, _request: Request) -> impl IntoResponse {
    tracing::debug!("Health check requested");
    Json(json!({ "status": "ok" }))
}

/// Shared utility to handle responses from Modal workers.
///
/// If the response is successful, it streams the body back to the client.
/// If unsuccessful, it returns a 502 Bad Gateway with details.
pub async fn handle_modal_response(res: reqwest::Response, default_content_type: &str) -> Response {
    if !res.status().is_success() {
        tracing::error!("Modal worker returned error: {}", res.status());
        let error_text = res.text().await.unwrap_or_default();
        tracing::error!("Modal worker error details: {}", error_text);
        return (
            StatusCode::BAD_GATEWAY,
            format!("Processing worker returned an error: {}", error_text),
        )
            .into_response();
    }

    let mut headers = HeaderMap::new();
    let ct = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or(default_content_type);

    if let Ok(value) = ct.parse() {
        headers.insert(header::CONTENT_TYPE, value);
    }

    let stream = res.bytes_stream();
    let body = Body::from_stream(stream);

    (headers, body).into_response()
}
