use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User model representing a registered user in the system.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub github_id: Option<String>,
    pub gitlab_id: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub credits: rust_decimal::Decimal,
    pub api_key: String,
    pub oauth_account_created_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Request payload for background removal via URL.
#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveBgRequest {
    /// URL of the image to process.
    pub url: String,
}

/// Supported models for image upscaling.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpscalerModel {
    /// Standard Real-ESRGAN model for high-quality upscaling.
    #[serde(rename = "RealESRGAN_x4plus")]
    RealEsrganX4plus,

    /// Alternative Real-ESRNet model.
    #[serde(rename = "RealESRNet_x4plus")]
    RealEsrnetX4plus,

    /// Specialized model for anime-style images.
    #[serde(rename = "RealESRGAN_x4plus_anime_6B")]
    RealEsrganX4plusAnime6B,

    /// Faster Real-ESRGAN model with 2x upscale factor.
    #[serde(rename = "RealESRGAN_x2plus")]
    RealEsrganX2plus,

    /// Versatile general-purpose model.
    #[serde(rename = "realesr-general-x4v3")]
    RealEsrGeneralX4v3,
}

impl std::fmt::Display for UpscalerModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RealEsrganX4plus => write!(f, "RealESRGAN_x4plus"),
            Self::RealEsrnetX4plus => write!(f, "RealESRNet_x4plus"),
            Self::RealEsrganX4plusAnime6B => write!(f, "RealESRGAN_x4plus_anime_6B"),
            Self::RealEsrganX2plus => write!(f, "RealESRGAN_x2plus"),
            Self::RealEsrGeneralX4v3 => write!(f, "realesr-general-x4v3"),
        }
    }
}

/// Request payload for image upscaling.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpscaleRequest {
    /// URL of the image to upscale.
    pub url: String,
    /// Optional model selection.
    pub model: Option<UpscalerModel>,
    /// Whether to apply face enhancement (GFPGAN).
    pub face_enhance: Option<bool>,
    /// Desired upscale factor (1-6).
    pub scale: Option<u32>,
}
