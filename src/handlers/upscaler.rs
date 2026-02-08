use crate::AppState;
use crate::models::{UpscaleRequest, UpscalerModel, User};
use axum::{
    Extension,
    extract::{FromRequest, Json, Multipart, Request, State},
    http::{StatusCode, header},
    response::Response,
};

/// Handler for image upscaling.
#[nijika_macros::price("0.02")]
pub async fn upscale(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    request: Request,
) -> Response {
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    let modal_url = &state.config.modal_upscaler_url;

    let result = if content_type.starts_with("application/json") {
        let Json(payload) = match Json::<UpscaleRequest>::from_request(request, &state.config).await
        {
            Ok(j) => j,
            Err(e) => {
                return (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)).into_response();
            }
        };

        if payload.scale.is_some_and(|scale| !(1..=6).contains(&scale)) {
            return (StatusCode::BAD_REQUEST, "Scale must be between 1 and 6").into_response();
        }

        state
            .http_client
            .post(modal_url)
            .json(&payload)
            .send()
            .await
    } else if content_type.starts_with("multipart/form-data") {
        let mut multipart = match Multipart::from_request(request, &state.config).await {
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
                        model =
                            serde_json::from_str::<UpscalerModel>(&format!("\"{}\"", text)).ok();
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
                return (StatusCode::BAD_REQUEST, "No image found in 'image' field")
                    .into_response();
            }
        };

        if scale.is_some_and(|s| !(1..=6).contains(&s)) {
            return (StatusCode::BAD_REQUEST, "Scale must be between 1 and 6").into_response();
        }

        let mut rb = state
            .http_client
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

        rb.send().await
    } else {
        return (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Content-Type must be application/json or multipart/form-data",
        )
            .into_response();
    };

    match result {
        Ok(res) => crate::handlers::handle_modal_response(res, "image/jpeg").await,
        Err(e) => {
            tracing::error!("Failed to call Modal worker: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to connect to processing worker",
            )
                .into_response()
        }
    }
}
