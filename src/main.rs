use std::net::SocketAddr;
use tokio::net::TcpListener;
use nijika_api::create_router;

/// Application entry point.
///
/// Initializes the environment, sets up tracing, creates the router,
/// and starts the Axum server.
///
/// # Environment Variables
///
/// * `HOST`: Interface to bind to (default: 127.0.0.1)
/// * `PORT`: Port to listen on (default: 3000)
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app = create_router();

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr_str = format!("{}:{}", host, port);
    
    let addr: SocketAddr = addr_str.parse().expect("Invalid HOST or PORT config");
    tracing::info!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
