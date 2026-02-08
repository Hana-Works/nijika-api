use askama::Template;
use axum::{
    Router,
    extract::{Query, Request, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use axum_governor::GovernorLayer;
use real::RealIpLayer;
use std::time::Duration;
use tower_http::trace::TraceLayer;
use tracing::Span;

pub mod auth_middleware;

use crate::AppState;
use crate::handlers::{auth, health_check, removebg, staff, upscaler};

use crate::models::User;
use axum::extract::Extension;
use axum::response::Redirect;
use axum_extra::extract::PrivateCookieJar;

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
    logs: Vec<crate::models::UsageLog>,
    logs_pagination: crate::models::PaginationMetadata,
    transactions: Vec<crate::models::Transaction>,
    tx_pagination: crate::models::PaginationMetadata,
}

#[derive(Debug, serde::Deserialize)]
struct DashboardQuery {
    l_page: Option<i64>,
    t_page: Option<i64>,
}

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
    let protected_routes = Router::new()
        .route("/dashboard", get(dashboard_page))
        .route("/auth/regenerate-api-key", post(auth::regenerate_api_key))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::session_auth,
        ));

    let moderator_routes = Router::new()
        .route("/staff/users", get(staff::users_list))
        .route("/staff/users/{id}", get(staff::user_detail))
        .route(
            "/staff/users/{id}/toggle-status",
            post(staff::toggle_user_status),
        )
        .route("/staff/logs", get(staff::audit_logs))
        .route_layer(axum::middleware::from_fn(auth_middleware::moderator_only));

    let admin_routes = Router::new()
        .route(
            "/staff/users/{id}/adjust-credits",
            post(staff::adjust_credits),
        )
        .route("/staff/users/{id}/change-role", post(staff::change_role))
        .route("/staff/system", get(staff::system_status))
        .route_layer(axum::middleware::from_fn(auth_middleware::admin_only));

    let staff_routes = Router::new()
        .merge(moderator_routes)
        .merge(admin_routes)
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::session_auth,
        ));

    Router::new()
        .route("/", get(landing_page))
        .route("/login", get(login_page))
        .route("/register", get(register_page))
        .merge(protected_routes)
        .merge(staff_routes)
        .route("/auth/github", get(auth::github_login))
        .route("/auth/github/callback", get(auth::github_callback))
        .route("/auth/gitlab", get(auth::gitlab_login))
        .route("/auth/gitlab/callback", get(auth::gitlab_callback))
        .route("/auth/logout", get(auth::logout))
        .route("/health", get(health_check).layer(GovernorLayer::default()))
        .nest(
            "/api",
            Router::new()
                .route("/removebg", post(removebg::remove_bg))
                .route("/upscale", post(upscaler::upscale))
                .layer(GovernorLayer::default())
                .route_layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware::api_key_auth,
                )),
        )
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
        .layer(RealIpLayer::default())
        .with_state(state)
}

async fn landing_page() -> LandingTemplate {
    LandingTemplate
}

async fn login_page(jar: PrivateCookieJar) -> Response {
    let user_id = jar
        .get("user_id")
        .and_then(|cookie| cookie.value().parse::<uuid::Uuid>().ok());

    tracing::debug!("Login page accessed. cookie user_id={:?}", user_id);

    if user_id.is_some() {
        tracing::debug!("User already logged in, redirecting from /login to /dashboard");
        return Redirect::to("/dashboard").into_response();
    }
    LoginTemplate.into_response()
}

async fn register_page(jar: PrivateCookieJar) -> Response {
    let user_id = jar
        .get("user_id")
        .and_then(|cookie| cookie.value().parse::<uuid::Uuid>().ok());

    if user_id.is_some() {
        return Redirect::to("/dashboard").into_response();
    }
    RegisterTemplate.into_response()
}

async fn dashboard_page(
    State(state): State<AppState>,
    Query(query): Query<DashboardQuery>,
    Extension(user): Extension<User>,
) -> DashboardTemplate {
    let page_size = 10;

    let logs_page = query.l_page.unwrap_or(1).max(1);
    let logs_offset = (logs_page - 1) * page_size;

    let total_logs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM usage_logs WHERE user_id = $1")
        .bind(user.id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let logs = sqlx::query_as::<_, crate::models::UsageLog>(
        "SELECT id, user_id, service, status, details, credits_used, created_at FROM usage_logs WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(user.id)
    .bind(page_size)
    .bind(logs_offset)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let logs_pagination = crate::models::PaginationMetadata {
        current_page: logs_page,
        total_pages: (total_logs + page_size - 1) / page_size,
        total_items: total_logs,
        page_size,
    };

    let tx_page = query.t_page.unwrap_or(1).max(1);
    let tx_offset = (tx_page - 1) * page_size;

    let total_tx: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions WHERE user_id = $1")
        .bind(user.id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let transactions = sqlx::query_as::<_, crate::models::Transaction>(
        "SELECT id, user_id, amount, type, description, created_at FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(user.id)
    .bind(page_size)
    .bind(tx_offset)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let tx_pagination = crate::models::PaginationMetadata {
        current_page: tx_page,
        total_pages: (total_tx + page_size - 1) / page_size,
        total_items: total_tx,
        page_size,
    };

    DashboardTemplate {
        user,
        logs,
        logs_pagination,
        transactions,
        tx_pagination,
    }
}
