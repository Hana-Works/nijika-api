use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use axum_extra::extract::cookie::Key;
use nijika_api::AppState;
use nijika_api::config::Config;
use nijika_api::create_router;
use sqlx::postgres::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_rate_limiting() {
    dotenvy::dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/unused".to_string());

    // Initialize rate limiter for testing
    lazy_limit::init_rate_limiter!(
        default: lazy_limit::RuleConfig::new(lazy_limit::Duration::seconds(1), 1),
        routes: [
            ("/health", lazy_limit::RuleConfig::new(lazy_limit::Duration::seconds(1), 1)),
        ]
    )
    .await;

    // Setup config
    let config = Arc::new(Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        modal_removebg_url: "http://localhost:8000".to_string(),
        modal_upscaler_url: "http://localhost:8001".to_string(),
        rate_limit_per_second: 1,
        rate_limit_burst: 1,
        database_url,
        github_client_id: "unused".to_string(),
        github_client_secret: "unused".to_string(),
        gitlab_client_id: "unused".to_string(),
        gitlab_client_secret: "unused".to_string(),
        base_url: "http://localhost:3000".to_string(),
        session_secret: "at-least-64-bytes-of-random-data-for-session-encryption-purposes-only"
            .to_string(),
    });

    // We use connect_lazy because we don't actually need a database for these tests
    // but the AppState requires a PgPool.
    let db = PgPool::connect_lazy(&config.database_url).unwrap();
    let state = AppState {
        config: config.clone(),
        db: db.clone(),
        http_client: reqwest::Client::new(),
        cookie_key: Key::from(config.session_secret.as_bytes()),
    };

    let app = create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 12345));

    // First request should succeed
    let request = Request::builder()
        .uri("/health")
        .extension(ConnectInfo(addr))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Second request immediate after should be rate limited
    let request = Request::builder()
        .uri("/health")
        .extension(ConnectInfo(addr))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    // Wait for 1.1 seconds (rate_limit_per_second is 1) to ensure replenishment
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

    // Third request should succeed again
    let request = Request::builder()
        .uri("/health")
        .extension(ConnectInfo(addr))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
