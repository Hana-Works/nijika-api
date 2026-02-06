use std::env;

/// Application configuration structure.
///
/// Holds all configuration parameters required by the application,
/// typically loaded from environment variables.
#[derive(Clone, Debug)]
pub struct Config {
    /// Interface to bind to (e.g., "127.0.0.1")
    pub host: String,
    /// Port to listen on (e.g., 3000)
    pub port: u16,
    /// URL for the background removal Modal worker
    pub modal_removebg_url: String,
    /// URL for the image upscaler Modal worker
    pub modal_upscaler_url: String,
    /// Rate limit: requests per second
    pub rate_limit_per_second: u64,
    /// Rate limit: burst size
    pub rate_limit_burst: u32,
}

impl Config {
    /// Loads configuration from environment variables.
    ///
    /// Uses default values if optional variables are missing.
    ///
    /// # Panics
    ///
    /// Panics if required variables are missing or if values are malformed.
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .expect("PORT must be a valid u16");
        let modal_removebg_url =
            env::var("MODAL_REMOVEBG_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
        let modal_upscaler_url =
            env::var("MODAL_UPSCALER_URL").unwrap_or_else(|_| "http://localhost:8001".to_string());
        let rate_limit_per_second = env::var("RATE_LIMIT_PER_SECOND")
            .unwrap_or_else(|_| "50".to_string())
            .parse::<u64>()
            .expect("RATE_LIMIT_PER_SECOND must be a valid u64");
        let rate_limit_burst = env::var("RATE_LIMIT_BURST")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u32>()
            .expect("RATE_LIMIT_BURST must be a valid u32");

        Self {
            host,
            port,
            modal_removebg_url,
            modal_upscaler_url,
            rate_limit_per_second,
            rate_limit_burst,
        }
    }
}
