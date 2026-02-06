use crate::config::Config;
use crate::models::RemoveBgRequest;
use axum::{
    body::Body,
    extract::{FromRequest, Json, Multipart, Request, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Handler for background removal.
///
/// Accepts either:
/// 1. `multipart/form-data` with an 'image' field (file upload).
/// 2. `application/json` with a 'url' field (image URL).
///
/// Forwards the request to a Modal worker for processing and returns the result.
pub async fn remove_bg(State(config): State<Arc<Config>>, request: Request) -> Response {
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    let modal_url = &config.modal_removebg_url;
    let client = reqwest::Client::new();

    if content_type.starts_with("application/json") {
        let Json(payload) = match Json::<RemoveBgRequest>::from_request(request, &config).await {
            Ok(j) => j,
            Err(e) => {
                return (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)).into_response();
            }
        };

        let res = match client.post(modal_url).json(&payload).send().await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Failed to call Modal worker: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to connect to processing worker",
                )
                    .into_response();
            }
        };

        handle_modal_response(res).await
    } else if content_type.starts_with("multipart/form-data") {
        let mut multipart = match Multipart::from_request(request, &config).await {
            Ok(m) => m,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid multipart request: {}", e),
                )
                    .into_response();
            }
        };

        let mut image_data = None;
        while let Some(field) = multipart.next_field().await.unwrap_or(None) {
            if field.name() == Some("image") {
                let Ok(bytes) = field.bytes().await else {
                    continue;
                };
                image_data = Some(bytes);
                break;
            }
        }

        let image_bytes = match image_data {
            Some(data) => data,
            None => {
                return (StatusCode::BAD_REQUEST, "No image found in 'image' field")
                    .into_response();
            }
        };

        let res = match client
            .post(modal_url)
            .body(image_bytes)
            .header("Content-Type", "application/octet-stream")
            .send()
            .await
        {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Failed to call Modal worker: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to connect to processing worker",
                )
                    .into_response();
            }
        };

        handle_modal_response(res).await
    } else {
        (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Content-Type must be application/json or multipart/form-data",
        )
            .into_response()
    }
}

async fn handle_modal_response(res: reqwest::Response) -> Response {
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
    headers.insert(header::CONTENT_TYPE, "image/png".parse().unwrap());

    let stream = res.bytes_stream();
    let body = Body::from_stream(stream);

    (headers, body).into_response()
}
