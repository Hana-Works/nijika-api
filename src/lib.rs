//! # Nijika API Library
//!
//! This library provides the core components for the Nijika API server.
//! It exports the main router creation function and exposes submodules for
//! handlers, models, and routes.

pub mod config;
pub mod handlers;
pub mod models;
pub mod routes;

pub use routes::create_router;

use config::Config;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: PgPool,
    pub http_client: reqwest::Client,
}
