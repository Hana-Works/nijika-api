use axum::{routing::get, Router};
use crate::handlers::health_check;

/// Creates the main application router.
///
/// This function registers all the routes and their corresponding handlers.
///
/// # Returns
///
/// A `Router` instance configured with all application routes.
pub fn create_router() -> Router {
    Router::new().route("/health", get(health_check))
}
