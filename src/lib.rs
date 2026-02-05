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
