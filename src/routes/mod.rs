use askama::Template; // Import the Askama Template trait
use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use std::sync::Arc;
use std::time::Duration;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::trace::TraceLayer;
use tracing::Span;

pub mod auth_middleware;

use crate::AppState;
use crate::handlers::{auth, health_check, removebg, upscaler};
use tower_sessions::{Expiry, SessionManagerLayer, cookie::SameSite};
use tower_sessions_sqlx_store::PostgresStore;

use crate::models::User;
use axum::extract::Extension;
use axum::response::Redirect;
use tower_sessions::Session;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "landing.html")]
struct LandingTemplate;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    user: User,
}

// Implement IntoResponse for all new templates
impl IntoResponse for LandingTemplate {
    fn into_response(self) -> axum::response::Response {
        match self.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", err),
            )
                .into_response(),
        }
    }
}

impl IntoResponse for LoginTemplate {
    fn into_response(self) -> axum::response::Response {
        match self.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", err),
            )
                .into_response(),
        }
    }
}

impl IntoResponse for RegisterTemplate {
    fn into_response(self) -> axum::response::Response {
        match self.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", err),
            )
                .into_response(),
        }
    }
}

impl IntoResponse for DashboardTemplate {
    fn into_response(self) -> axum::response::Response {
        match self.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", err),
            )
                .into_response(),
        }
    }
}

/// Creates the main application router.
///
/// This function registers all the routes and their corresponding handlers.
///
/// # Returns
///
/// A `Router` instance configured with all application routes.
pub fn create_router(state: AppState) -> Router {
    let session_store = PostgresStore::new(state.db.clone());
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(
            tower_sessions::cookie::time::Duration::hours(24),
        ));

    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(state.config.rate_limit_per_second)
            .burst_size(state.config.rate_limit_burst)
            .finish()
            .unwrap(),
    );

    let protected_routes = Router::new()
        .route("/dashboard", get(dashboard_page))
        .route("/auth/regenerate-api-key", post(auth::regenerate_api_key))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::session_auth,
        ));

    Router::new()
        .route("/", get(landing_page))
        .route("/login", get(login_page))
        .route("/register", get(register_page))
        .merge(protected_routes)
        .route("/auth/github", get(auth::github_login))
        .route("/auth/github/callback", get(auth::github_callback))
        .route("/auth/gitlab", get(auth::gitlab_login))
        .route("/auth/gitlab/callback", get(auth::gitlab_callback))
        .route("/auth/logout", get(auth::logout))
        .route("/health", get(health_check))
        .nest(
            "/api",
            Router::new()
                .route("/removebg", post(removebg::remove_bg))
                .route("/upscale", post(upscaler::upscale))
                .route_layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware::api_key_auth,
                )),
        )
        .layer(GovernorLayer::new(governor_conf))
        .layer(session_layer)
        .layer(
            TraceLayer::new_for_http()
                .on_request(|request: &Request<_>, _span: &Span| {
                    tracing::info!(
                        "started processing request: method={} uri={}",
                        request.method(),
                        request.uri()
                    );
                })
                .on_response(
                    |response: &axum::response::Response, latency: Duration, _span: &Span| {
                        tracing::info!(
                            "finished processing request: status={} latency={:?}",
                            response.status(),
                            latency
                        );
                    },
                ),
        )
        .with_state(state)
}

async fn landing_page() -> LandingTemplate {
    LandingTemplate
}

async fn login_page(session: Session) -> Response {
    if session
        .get::<Uuid>("user_id")
        .await
        .unwrap_or(None)
        .is_some()
    {
        return Redirect::to("/dashboard").into_response();
    }
    LoginTemplate.into_response()
}

async fn register_page(session: Session) -> Response {
    if session
        .get::<Uuid>("user_id")
        .await
        .unwrap_or(None)
        .is_some()
    {
        return Redirect::to("/dashboard").into_response();
    }
    RegisterTemplate.into_response()
}

async fn dashboard_page(Extension(user): Extension<User>) -> DashboardTemplate {
    DashboardTemplate { user }
}
