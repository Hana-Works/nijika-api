use crate::AppState;
use crate::models::User;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub async fn api_key_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let api_key = req
        .headers()
        .get("X-API-Key")
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE api_key = $1")
        .bind(api_key)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if user.credits <= rust_decimal::Decimal::ZERO {
        return Err(StatusCode::PAYMENT_REQUIRED);
    }

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

pub async fn session_auth(
    State(state): State<AppState>,
    session: tower_sessions::Session,
    mut req: Request,
    next: Next,
) -> Result<Response, axum::response::Redirect> {
    let user_id: Option<uuid::Uuid> = session.get("user_id").await.unwrap_or(None);

    if let Some(uid) = user_id {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(uid)
            .fetch_one(&state.db)
            .await
            .map_err(|_| axum::response::Redirect::to("/login"))?;

        req.extensions_mut().insert(user);
        Ok(next.run(req).await)
    } else {
        Err(axum::response::Redirect::to("/login"))
    }
}
