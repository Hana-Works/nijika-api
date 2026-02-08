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
    /// Database connection URL
    pub database_url: String,
    /// GitHub OAuth Client ID
    pub github_client_id: String,
    /// GitHub OAuth Client Secret
    pub github_client_secret: String,
    /// GitLab OAuth Client ID
    pub gitlab_client_id: String,
    /// GitLab OAuth Client Secret
    pub gitlab_client_secret: String,
    /// Base URL for OAuth callbacks
    pub base_url: String,
    /// Secret key for cookie encryption
    pub session_secret: String,
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
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let github_client_id = env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set");
        let github_client_secret =
            env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set");
        let gitlab_client_id = env::var("GITLAB_CLIENT_ID").expect("GITLAB_CLIENT_ID must be set");
        let gitlab_client_secret =
            env::var("GITLAB_CLIENT_SECRET").expect("GITLAB_CLIENT_SECRET must be set");
        let base_url = env::var("BASE_URL").unwrap_or_else(|_| format!("http://{}:{}", host, port));
        let session_secret = env::var("SESSION_SECRET").unwrap_or_else(|_| {
            "at-least-64-bytes-of-random-data-for-session-encryption-purposes-only".to_string()
        });

        Self {
            host,
            port,
            modal_removebg_url,
            modal_upscaler_url,
            rate_limit_per_second,
            rate_limit_burst,
            database_url,
            github_client_id,
            github_client_secret,
            gitlab_client_id,
            gitlab_client_secret,
            base_url,
            session_secret,
        }
    }
}
