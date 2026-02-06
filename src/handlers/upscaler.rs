use crate::config::Config;
use crate::models::{UpscaleRequest, UpscalerModel};
use axum::{
    body::Body,
    extract::{FromRequest, Json, Multipart, Request, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Handler for image upscaling.
///
/// Accepts either:
/// 1. `multipart/form-data` with an 'image' field (file upload) and optional parameters.
/// 2. `application/json` with a 'url' field (image URL) and optional parameters.
///
/// Forwards the request to a Modal worker for processing and returns the result.
pub async fn upscale(State(config): State<Arc<Config>>, request: Request) -> Response {
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    let modal_url = &config.modal_upscaler_url;
    let client = reqwest::Client::new();

    if content_type.starts_with("application/json") {
        let Json(payload) = match Json::<UpscaleRequest>::from_request(request, &config).await {
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
        let mut model: Option<UpscalerModel> = None;
        let mut scale = None;
        let mut face_enhance = None;

        while let Some(field) = multipart.next_field().await.unwrap_or(None) {
            let name = field.name().unwrap_or("").to_string();
            match name.as_str() {
                "image" => {
                    if let Ok(bytes) = field.bytes().await {
                        image_data = Some(bytes);
                    }
                }
                "model" => {
                    if let Ok(text) = field.text().await {
                        if let Ok(m) =
                            serde_json::from_str::<UpscalerModel>(&format!("\"{}\"", text))
                        {
                            model = Some(m);
                        }
                    }
                }
                "scale" => {
                    if let Ok(text) = field.text().await {
                        scale = text.parse::<u32>().ok();
                    }
                }
                "face_enhance" => {
                    if let Ok(text) = field.text().await {
                        face_enhance = Some(text == "true" || text == "1");
                    }
                }
                _ => {}
            }
        }

        let image_bytes = match image_data {
            Some(data) => data,
            None => {
                return (StatusCode::BAD_REQUEST, "No image found in 'image' field").into_response();
            }
        };

        let mut rb = client
            .post(modal_url)
            .body(image_bytes)
            .header("Content-Type", "application/octet-stream");

        if let Some(m) = model {
            rb = rb.header("X-Model", m.to_string());
        }
        if let Some(s) = scale {
            rb = rb.header("X-Scale", s.to_string());
        }
        if let Some(f) = face_enhance {
            rb = rb.header("X-Face-Enhance", f.to_string());
        }

        let res = match rb.send().await {
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
    let ct = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg");

    headers.insert(header::CONTENT_TYPE, ct.parse().unwrap());

    let stream = res.bytes_stream();
    let body = Body::from_stream(stream);

    (headers, body).into_response()
}
