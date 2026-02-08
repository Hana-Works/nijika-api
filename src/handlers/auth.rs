use crate::AppState;
use crate::models::User;
use axum::{
    Extension,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use chrono::{DateTime, Months, Utc};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl, basic::BasicClient,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use tower_sessions::Session;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

fn create_github_client(state: &AppState) -> BasicClient {
    BasicClient::new(
        ClientId::new(state.config.github_client_id.clone()),
        Some(ClientSecret::new(state.config.github_client_secret.clone())),
        AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap()),
    )
    .set_redirect_uri(
        RedirectUrl::new(format!("{}/auth/github/callback", state.config.base_url)).unwrap(),
    )
}

fn create_gitlab_client(state: &AppState) -> BasicClient {
    BasicClient::new(
        ClientId::new(state.config.gitlab_client_id.clone()),
        Some(ClientSecret::new(state.config.gitlab_client_secret.clone())),
        AuthUrl::new("https://gitlab.com/oauth/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://gitlab.com/oauth/token".to_string()).unwrap()),
    )
    .set_redirect_uri(
        RedirectUrl::new(format!("{}/auth/gitlab/callback", state.config.base_url)).unwrap(),
    )
}

pub async fn github_login(State(state): State<AppState>, session: Session) -> impl IntoResponse {
    let client = create_github_client(&state);

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .add_scope(Scope::new("read:user".to_string()))
        .url();

    if let Err(e) = session
        .insert("csrf_token", csrf_token.secret().to_string())
        .await
    {
        tracing::error!("Failed to insert CSRF token into session: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response();
    }

    Redirect::to(auth_url.as_str()).into_response()
}

#[derive(Debug, Deserialize)]
struct GithubUser {
    id: i64,
    login: String,
    email: Option<String>,
    created_at: DateTime<Utc>,
}

pub async fn github_callback(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<AuthRequest>,
) -> impl IntoResponse {
    let stored_csrf: String = session
        .get("csrf_token")
        .await
        .unwrap_or_default()
        .unwrap_or_default();

    if query.state != stored_csrf {
        return (StatusCode::BAD_REQUEST, "Invalid CSRF token").into_response();
    }

    let client = create_github_client(&state);

    let token_result = match client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(|request| custom_http_client(state.http_client.clone(), request))
        .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to exchange GitHub token: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Authentication failed").into_response();
        }
    };

    let github_user_res = state
        .http_client
        .get("https://api.github.com/user")
        .header("User-Agent", "nijika-api")
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await;

    let github_user: GithubUser = match github_user_res {
        Ok(res) => match res.json().await {
            Ok(u) => u,
            Err(e) => {
                tracing::error!("Failed to parse GitHub user: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Authentication failed")
                    .into_response();
            }
        },
        Err(e) => {
            tracing::error!("Failed to fetch GitHub user: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Authentication failed").into_response();
        }
    };

    if !is_account_old_enough(github_user.created_at) {
        return (
            StatusCode::FORBIDDEN,
            "Account must be at least 1 month old to prevent abuse.",
        )
            .into_response();
    }

    let github_id = github_user.id.to_string();
    let current_user_id: Option<Uuid> = session.get("user_id").await.unwrap_or(None);

    let user_by_github_id = sqlx::query_as::<_, User>("SELECT * FROM users WHERE github_id = $1")
        .bind(&github_id)
        .fetch_optional(&state.db)
        .await;

    let user_by_github_id = match user_by_github_id {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Database error fetching user by github_id: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let user = match (current_user_id, user_by_github_id) {
        // Case 1: Logged in, and this GitHub account is already linked to THIS user.
        (Some(uid), Some(u)) if u.id == uid => u,

        // Case 2: Logged in, but this GitHub account is linked to ANOTHER user.
        (Some(_), Some(_)) => {
            return (
                StatusCode::BAD_REQUEST,
                "GitHub account already linked to another user",
            )
                .into_response();
        }

        // Case 3: Logged in, and this GitHub account is NOT linked to anyone yet.
        (Some(uid), None) => {
            let link_res = sqlx::query_as::<_, User>(
                "UPDATE users SET github_id = $1 WHERE id = $2 RETURNING *",
            )
            .bind(&github_id)
            .bind(uid)
            .fetch_one(&state.db)
            .await;

            match link_res {
                Ok(u) => u,
                Err(e) => {
                    tracing::error!("Database error linking github_id: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
                }
            }
        }

        // Case 4: NOT logged in, but this GitHub account is already linked to a user.
        (None, Some(u)) => u,

        // Case 5: NOT logged in, and this GitHub account is NOT linked to anyone.
        (None, None) => {
            let user_by_email = if let Some(ref email) = github_user.email {
                sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
                    .bind(email)
                    .fetch_optional(&state.db)
                    .await
            } else {
                Ok(None)
            };

            let user_by_email = match user_by_email {
                Ok(u) => u,
                Err(e) => {
                    tracing::error!("Database error fetching user by email: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
                }
            };

            match user_by_email {
                // Automatic linking by email
                Some(u) => {
                    let link_res = sqlx::query_as::<_, User>(
                        "UPDATE users SET github_id = $1 WHERE id = $2 RETURNING *",
                    )
                    .bind(&github_id)
                    .bind(u.id)
                    .fetch_one(&state.db)
                    .await;

                    match link_res {
                        Ok(u) => u,
                        Err(e) => {
                            tracing::error!("Database error linking github_id by email: {}", e);
                            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
                                .into_response();
                        }
                    }
                }
                // Create new user
                None => {
                    let api_key = Uuid::new_v4().to_string();
                    let insert_res = sqlx::query_as::<_, User>(
                        "INSERT INTO users (github_id, username, email, api_key, oauth_account_created_at, credits) 
                         VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
                    )
                    .bind(&github_id)
                    .bind(&github_user.login)
                    .bind(&github_user.email)
                    .bind(&api_key)
                    .bind(github_user.created_at)
                    .bind(Decimal::new(5000000000, 8)) // 50.00000000
                    .fetch_one(&state.db)
                    .await;

                    match insert_res {
                        Ok(u) => u,
                        Err(e) => {
                            tracing::error!("Database error creating user: {}", e);
                            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
                                .into_response();
                        }
                    }
                }
            }
        }
    };

    if let Err(e) = session.insert("user_id", user.id).await {
        tracing::error!("Failed to insert user_id into session: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response();
    }
    Redirect::to("/dashboard").into_response()
}

pub async fn custom_http_client(
    client: reqwest::Client,
    request: oauth2::HttpRequest,
) -> Result<oauth2::HttpResponse, oauth2::reqwest::Error<reqwest::Error>> {
    let method = match request.method {
        oauth2::http::Method::GET => reqwest::Method::GET,
        oauth2::http::Method::POST => reqwest::Method::POST,
        oauth2::http::Method::PUT => reqwest::Method::PUT,
        oauth2::http::Method::DELETE => reqwest::Method::DELETE,
        oauth2::http::Method::HEAD => reqwest::Method::HEAD,
        oauth2::http::Method::OPTIONS => reqwest::Method::OPTIONS,
        oauth2::http::Method::CONNECT => reqwest::Method::CONNECT,
        oauth2::http::Method::PATCH => reqwest::Method::PATCH,
        oauth2::http::Method::TRACE => reqwest::Method::TRACE,
        _ => reqwest::Method::from_bytes(request.method.as_str().as_bytes()).unwrap(),
    };

    let mut request_builder = client
        .request(method, request.url.as_str())
        .body(request.body);

    for (name, value) in &request.headers {
        request_builder = request_builder.header(name.as_str(), value.as_bytes());
    }

    let response = request_builder
        .send()
        .await
        .map_err(oauth2::reqwest::Error::Reqwest)?;

    let status_code = oauth2::http::StatusCode::from_u16(response.status().as_u16()).unwrap();
    let mut headers = oauth2::http::HeaderMap::new();
    for (name, value) in response.headers() {
        headers.insert(
            oauth2::http::header::HeaderName::from_bytes(name.as_str().as_bytes()).unwrap(),
            oauth2::http::header::HeaderValue::from_bytes(value.as_bytes()).unwrap(),
        );
    }

    let body = response
        .bytes()
        .await
        .map_err(oauth2::reqwest::Error::Reqwest)?
        .to_vec();

    Ok(oauth2::HttpResponse {
        status_code,
        headers,
        body,
    })
}

pub async fn gitlab_login(State(state): State<AppState>, session: Session) -> impl IntoResponse {
    let client = create_gitlab_client(&state);

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read_user".to_string()))
        .url();

    if let Err(e) = session
        .insert("csrf_token", csrf_token.secret().to_string())
        .await
    {
        tracing::error!("Failed to insert CSRF token into session: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response();
    }

    Redirect::to(auth_url.as_str()).into_response()
}

#[derive(Debug, Deserialize)]
struct GitlabUser {
    id: i64,
    username: String,
    email: Option<String>,
    created_at: DateTime<Utc>,
}

pub async fn gitlab_callback(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<AuthRequest>,
) -> impl IntoResponse {
    let stored_csrf: String = session
        .get("csrf_token")
        .await
        .unwrap_or_default()
        .unwrap_or_default();
    if query.state != stored_csrf {
        return (StatusCode::BAD_REQUEST, "Invalid CSRF token").into_response();
    }

    let client = create_gitlab_client(&state);

    let token_result = match client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(|request| custom_http_client(state.http_client.clone(), request))
        .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to exchange GitLab token: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Authentication failed").into_response();
        }
    };

    let gitlab_user_res = state
        .http_client
        .get("https://gitlab.com/api/v4/user")
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await;

    let gitlab_user: GitlabUser = match gitlab_user_res {
        Ok(res) => match res.json().await {
            Ok(u) => u,
            Err(e) => {
                tracing::error!("Failed to parse GitLab user: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Authentication failed")
                    .into_response();
            }
        },
        Err(e) => {
            tracing::error!("Failed to fetch GitLab user: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Authentication failed").into_response();
        }
    };

    if !is_account_old_enough(gitlab_user.created_at) {
        return (
            StatusCode::FORBIDDEN,
            "Account must be at least 1 month old to prevent abuse.",
        )
            .into_response();
    }

    let gitlab_id = gitlab_user.id.to_string();
    let current_user_id: Option<Uuid> = session.get("user_id").await.unwrap_or(None);

    let user_by_gitlab_id = sqlx::query_as::<_, User>("SELECT * FROM users WHERE gitlab_id = $1")
        .bind(&gitlab_id)
        .fetch_optional(&state.db)
        .await;

    let user_by_gitlab_id = match user_by_gitlab_id {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Database error fetching user by gitlab_id: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let user = match (current_user_id, user_by_gitlab_id) {
        // Case 1: Logged in, and this GitLab account is already linked to THIS user.
        (Some(uid), Some(u)) if u.id == uid => u,

        // Case 2: Logged in, but this GitLab account is linked to ANOTHER user.
        (Some(_), Some(_)) => {
            return (
                StatusCode::BAD_REQUEST,
                "GitLab account already linked to another user",
            )
                .into_response();
        }

        // Case 3: Logged in, and this GitLab account is NOT linked to anyone yet.
        (Some(uid), None) => {
            let link_res = sqlx::query_as::<_, User>(
                "UPDATE users SET gitlab_id = $1 WHERE id = $2 RETURNING *",
            )
            .bind(&gitlab_id)
            .bind(uid)
            .fetch_one(&state.db)
            .await;

            match link_res {
                Ok(u) => u,
                Err(e) => {
                    tracing::error!("Database error linking gitlab_id: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
                }
            }
        }

        // Case 4: NOT logged in, but this GitLab account is already linked to a user.
        (None, Some(u)) => u,

        // Case 5: NOT logged in, and this GitLab account is NOT linked to anyone.
        (None, None) => {
            let user_by_email = if let Some(ref email) = gitlab_user.email {
                sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
                    .bind(email)
                    .fetch_optional(&state.db)
                    .await
            } else {
                Ok(None)
            };

            let user_by_email = match user_by_email {
                Ok(u) => u,
                Err(e) => {
                    tracing::error!("Database error fetching user by email: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
                }
            };

            match user_by_email {
                // Automatic linking by email
                Some(u) => {
                    let link_res = sqlx::query_as::<_, User>(
                        "UPDATE users SET gitlab_id = $1 WHERE id = $2 RETURNING *",
                    )
                    .bind(&gitlab_id)
                    .bind(u.id)
                    .fetch_one(&state.db)
                    .await;

                    match link_res {
                        Ok(u) => u,
                        Err(e) => {
                            tracing::error!("Database error linking gitlab_id by email: {}", e);
                            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
                                .into_response();
                        }
                    }
                }
                // Create new user
                None => {
                    let api_key = Uuid::new_v4().to_string();
                    let insert_res = sqlx::query_as::<_, User>(
                        "INSERT INTO users (gitlab_id, username, email, api_key, oauth_account_created_at, credits) 
                         VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
                    )
                    .bind(&gitlab_id)
                    .bind(&gitlab_user.username)
                    .bind(&gitlab_user.email)
                    .bind(&api_key)
                    .bind(gitlab_user.created_at)
                    .bind(Decimal::new(5000000000, 8)) // 50.00000000
                    .fetch_one(&state.db)
                    .await;

                    match insert_res {
                        Ok(u) => u,
                        Err(e) => {
                            tracing::error!("Database error creating user: {}", e);
                            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
                                .into_response();
                        }
                    }
                }
            }
        }
    };

    if let Err(e) = session.insert("user_id", user.id).await {
        tracing::error!("Failed to insert user_id into session: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response();
    }
    Redirect::to("/dashboard").into_response()
}

pub async fn logout(session: Session) -> impl IntoResponse {
    session.clear().await;

    Redirect::to("/")
}

pub async fn regenerate_api_key(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    let new_api_key = Uuid::new_v4().to_string();

    let result = sqlx::query("UPDATE users SET api_key = $1 WHERE id = $2")
        .bind(new_api_key)
        .bind(user.id)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => {
            tracing::info!("Successfully regenerated API key for user {}", user.id);
            Redirect::to("/dashboard").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to regenerate API key for user {}: {}", user.id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to regenerate API key: {}", e),
            )
                .into_response()
        }
    }
}

fn is_account_old_enough(created_at: DateTime<Utc>) -> bool {
    let one_month_ago = Utc::now()
        .checked_sub_months(Months::new(1))
        .unwrap_or_else(Utc::now);
    created_at <= one_month_ago
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_is_account_old_enough() {
        let old_account = Utc::now() - Duration::days(32);
        let new_account = Utc::now() - Duration::days(28);

        assert!(is_account_old_enough(old_account));
        assert!(!is_account_old_enough(new_account));
    }
}
