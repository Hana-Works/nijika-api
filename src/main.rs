use nijika_api::{config::Config, create_router};
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

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Arc::new(Config::from_env());
    let app = create_router(config.clone());

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
