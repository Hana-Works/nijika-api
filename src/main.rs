use lazy_limit::{Duration, RuleConfig, init_rate_limiter};
use nijika_api::{AppState, config::Config, create_router};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Application entry point.
///
/// Initializes the environment, sets up tracing, creates the router,
/// and starts the Axum server.
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    init_rate_limiter!(
        default: RuleConfig::new(Duration::seconds(1), 5),
        max_memory: Some(64 * 1024 * 1024),
        routes: [
            ("/health", RuleConfig::new(Duration::seconds(1), 100)),
            ("/api/", RuleConfig::new(Duration::seconds(1), 5).match_prefix(true)),
        ]
    )
    .await;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Arc::new(Config::from_env());

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    let cookie_key = axum_extra::extract::cookie::Key::from(config.session_secret.as_bytes());

    let state = AppState {
        config: config.clone(),
        db,
        http_client: reqwest::Client::new(),
        cookie_key,
    };

    let app = create_router(state);

    let addr_str = format!("{}:{}", config.host, config.port);
    let addr: SocketAddr = addr_str.parse().expect("Invalid HOST or PORT config");

    tracing::info!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
