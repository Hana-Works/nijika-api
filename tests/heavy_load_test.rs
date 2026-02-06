use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use nijika_api::config::Config;
use nijika_api::create_router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::JoinSet;
use tower::ServiceExt;

#[tokio::test]
async fn test_heavy_load_health_check() {
    // Configure with a known limit to predict behavior
    // 100 requests per second, burst of 50
    let config = Arc::new(Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        modal_removebg_url: "http://localhost:8000".to_string(),
        modal_upscaler_url: "http://localhost:8001".to_string(),
        rate_limit_per_second: 100,
        rate_limit_burst: 50,
    });

    let app = create_router(config);

    // Simulate 200 concurrent requests
    let total_requests = 200;
    let mut set = JoinSet::new();

    // Use a fixed address for all requests to trigger IP-based rate limiting if applicable
    // or use different ports to simulate different clients if we wanted to test global throughput.
    // Governor usually defaults to PeerIpKeyExtractor.
    // Let's simulate a single IP flooding the server.
    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));

    for _ in 0..total_requests {
        let app = app.clone();
        set.spawn(async move {
            let request = Request::builder()
                .uri("/health")
                .extension(ConnectInfo(addr))
                .body(Body::empty())
                .unwrap();

            // We use oneshot, which clones the service for each request in the test loop usually,
            // but here we are inside a spawn.
            // Note: Router::oneshot consumes the router. We cloned it above.
            match app.oneshot(request).await {
                Ok(response) => response.status(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        });
    }

    let mut success_count = 0;
    let mut rate_limit_count = 0;
    let mut error_count = 0;

    while let Some(res) = set.join_next().await {
        match res {
            Ok(status) => match status {
                StatusCode::OK => success_count += 1,
                StatusCode::TOO_MANY_REQUESTS => rate_limit_count += 1,
                _ => error_count += 1,
            },
            Err(e) => {
                println!("Task join error: {:?}", e);
                error_count += 1;
            }
        }
    }

    println!("Load Test Results:");
    println!("Total Requests: {}", total_requests);
    println!("Success (200): {}", success_count);
    println!("Rate Limited (429): {}", rate_limit_count);
    println!("Errors: {}", error_count);

    // Assertions
    assert_eq!(
        success_count + rate_limit_count + error_count,
        total_requests
    );
    assert_eq!(error_count, 0, "Should not have any internal server errors");

    // With a burst of 50, we expect at least 50 successes.
    // The rest might be rate limited depending on execution speed.
    assert!(
        success_count >= 50,
        "Should allow at least the burst amount"
    );

    // Since we sent 200 requests with a burst of 50, we expect some 429s
    assert!(rate_limit_count > 0, "Should have triggered rate limiting");
}
