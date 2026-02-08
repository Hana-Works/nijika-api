use crate::AppState;
use crate::models::User;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::PrivateCookieJar;
use axum_extra::extract::cookie::Cookie;

pub async fn api_key_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, Response> {
    let api_key = req
        .headers()
        .get("X-API-Key")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, github_id, gitlab_id, email, username, avatar_url, credits, role, is_active, api_key, oauth_account_created_at, created_at FROM users WHERE api_key = $1"
    )
        .bind(api_key)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
        .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;

    if !user.is_active {
        return Err(StatusCode::FORBIDDEN.into_response());
    }

    if user.credits <= rust_decimal::Decimal::ZERO {
        return Err(StatusCode::PAYMENT_REQUIRED.into_response());
    }

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

pub async fn session_auth(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, Response> {
    let user_id = jar
        .get("user_id")
        .and_then(|cookie| cookie.value().parse::<uuid::Uuid>().ok());

    tracing::debug!("Cookie auth check: user_id={:?}", user_id);

    if let Some(uid) = user_id {
        match sqlx::query_as::<_, User>(
            "SELECT id, github_id, gitlab_id, email, username, avatar_url, credits, role, is_active, api_key, oauth_account_created_at, created_at FROM users WHERE id = $1"
        )
            .bind(uid)
            .fetch_one(&state.db)
            .await
        {
            Ok(user) => {
                if !user.is_active {
                    let jar = jar.remove(Cookie::build("user_id").path("/").build());
                    return Err((jar, axum::response::Redirect::to("/login")).into_response());
                }
                req.extensions_mut().insert(user);
                Ok(next.run(req).await)
            }
            Err(e) => {
                tracing::error!("Failed to fetch user from DB in cookie_auth: {}", e);
                let jar = jar.remove(Cookie::build("user_id").path("/").build());
                Err((jar, axum::response::Redirect::to("/login")).into_response())
            }
        }
    } else {
        tracing::debug!("No user_id in cookies, redirecting to /login");
        Err(axum::response::Redirect::to("/login").into_response())
    }
}

pub async fn admin_only(req: Request, next: Next) -> Result<Response, Response> {
    let user = req
        .extensions()
        .get::<User>()
        .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;

    if !user.is_admin() {
        return Err(StatusCode::FORBIDDEN.into_response());
    }

    Ok(next.run(req).await)
}

pub async fn moderator_only(req: Request, next: Next) -> Result<Response, Response> {
    let user = req
        .extensions()
        .get::<User>()
        .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;

    if !user.is_moderator() {
        return Err(StatusCode::FORBIDDEN.into_response());
    }

    Ok(next.run(req).await)
}
