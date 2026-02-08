use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use nijika_api::AppState;
use nijika_api::config::Config;
use nijika_api::create_router;
use sqlx::postgres::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_rate_limiting() {
    // Setup config with very low rate limits for testing
    let config = Arc::new(Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        modal_removebg_url: "http://localhost:8000".to_string(),
        modal_upscaler_url: "http://localhost:8001".to_string(),
        rate_limit_per_second: 1,
        rate_limit_burst: 1,
        database_url: "postgres://localhost/unused".to_string(),
        github_client_id: "unused".to_string(),
        github_client_secret: "unused".to_string(),
        gitlab_client_id: "unused".to_string(),
        gitlab_client_secret: "unused".to_string(),
        base_url: "http://localhost:3000".to_string(),
    });

    // We use connect_lazy because we don't actually need a database for these tests
    // but the AppState requires a PgPool.
    let db = PgPool::connect_lazy(&config.database_url).unwrap();
    let state = AppState {
        config: config.clone(),
        db,
        http_client: reqwest::Client::new(),
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

    // Wait for 1 second (rate_limit_per_second is 1)
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Third request should succeed again
    let request = Request::builder()
        .uri("/health")
        .extension(ConnectInfo(addr))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
