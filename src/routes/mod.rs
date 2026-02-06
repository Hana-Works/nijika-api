use axum::{
    Router,
    extract::Request,
    routing::{get, post},
};
use std::sync::Arc;
use std::time::Duration;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::trace::TraceLayer;
use tracing::Span;

use crate::config::Config;
use crate::handlers::{health_check, removebg, upscaler};

/// Creates the main application router.
///
/// This function registers all the routes and their corresponding handlers.
///
/// # Returns
///
/// A `Router` instance configured with all application routes.
pub fn create_router(config: Arc<Config>) -> Router {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(config.rate_limit_per_second)
            .burst_size(config.rate_limit_burst)
            .finish()
            .unwrap(),
    );

    Router::new()
        .route("/health", get(health_check))
        .route("/removebg", post(removebg::remove_bg))
        .route("/upscale", post(upscaler::upscale))
        .layer(GovernorLayer::new(governor_conf))
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
        .with_state(config)
}
