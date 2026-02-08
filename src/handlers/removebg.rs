use crate::AppState;
use crate::models::{RemoveBgRequest, User};
use axum::{
    Extension,
    extract::{FromRequest, Json, Multipart, Request, State},
    http::{StatusCode, header},
    response::Response,
};

/// Handler for background removal.
#[nijika_macros::price("0.01")]
pub async fn remove_bg(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    request: Request,
) -> Response {
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    let modal_url = &state.config.modal_removebg_url;

    let result = if content_type.starts_with("application/json") {
        let Json(payload) = match Json::<RemoveBgRequest>::from_request(request, &state.config)
            .await
        {
            Ok(j) => j,
            Err(e) => {
                return (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)).into_response();
            }
        };

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

        state
            .http_client
            .post(modal_url)
            .body(image_bytes)
            .header("Content-Type", "application/octet-stream")
            .send()
            .await
    } else {
        return (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Content-Type must be application/json or multipart/form-data",
        )
            .into_response();
    };

    match result {
        Ok(res) => crate::handlers::handle_modal_response(res, "image/png").await,
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
