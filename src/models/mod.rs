use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User role representing the authorization level of a user.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    /// Administrator with full access.
    Admin,
    /// Moderator with elevated access.
    Moderator,
    /// Regular user with standard access.
    User,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Moderator => "moderator",
            Self::User => "user",
        }
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "admin"),
            Self::Moderator => write!(f, "moderator"),
            Self::User => write!(f, "user"),
        }
    }
}

/// User model representing a registered user in the system.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub github_id: Option<String>,
    pub gitlab_id: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub credits: rust_decimal::Decimal,
    pub role: UserRole,
    pub is_active: bool,
    pub api_key: String,
    pub oauth_account_created_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl User {
    /// Returns the credits as a clean string without unnecessary trailing zeros.
    pub fn formatted_credits(&self) -> String {
        self.credits.normalize().to_string()
    }

    /// Checks if the user has administrator privileges.
    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin)
    }

    /// Checks if the user has moderator or higher privileges.
    pub fn is_moderator(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::Moderator)
    }

    /// Returns the role as a string.
    pub fn role_name(&self) -> String {
        self.role.to_string()
    }
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

/// Usage log entry representing a service call by a user.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub service: String,
    pub status: String,
    pub details: Option<String>,
    pub credits_used: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
}

impl UsageLog {
    /// Returns the credits used as a clean string.
    pub fn formatted_credits(&self) -> String {
        self.credits_used.normalize().to_string()
    }

    /// Returns a human-friendly representation of the creation time.
    pub fn formatted_time(&self) -> String {
        self.created_at.format("%b %d, %H:%M").to_string()
    }
}

/// Transaction type representing the nature of the credit change.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "transaction_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// Credits spent for using a service.
    Charge,
    /// Credits refunded due to a failed service call.
    Refund,
    /// Credits purchased by the user.
    Deposit,
    /// Credits given as a bonus (e.g., registration).
    Bonus,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Charge => write!(f, "charge"),
            Self::Refund => write!(f, "refund"),
            Self::Deposit => write!(f, "deposit"),
            Self::Bonus => write!(f, "bonus"),
        }
    }
}

impl PartialEq<&str> for TransactionType {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Self::Charge => *other == "charge",
            Self::Refund => *other == "refund",
            Self::Deposit => *other == "deposit",
            Self::Bonus => *other == "bonus",
        }
    }
}

/// Transaction model representing a credit change for a user.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub r#type: TransactionType,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    /// Returns the amount as a clean string.
    pub fn formatted_amount(&self) -> String {
        self.amount.normalize().to_string()
    }

    /// Returns a human-friendly representation of the creation time.
    pub fn formatted_time(&self) -> String {
        self.created_at.format("%b %d, %H:%M").to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMetadata {
    pub current_page: i64,
    pub total_pages: i64,
    pub total_items: i64,
    pub page_size: i64,
}

impl PaginationMetadata {
    pub fn has_previous(&self) -> bool {
        self.current_page > 1
    }

    pub fn has_next(&self) -> bool {
        self.current_page < self.total_pages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_user_role_display() {
        assert_eq!(UserRole::Admin.to_string(), "admin");
        assert_eq!(UserRole::Moderator.to_string(), "moderator");
        assert_eq!(UserRole::User.to_string(), "user");
    }

    #[test]
    fn test_user_is_admin_moderator() {
        let mut user = User {
            id: Uuid::new_v4(),
            github_id: None,
            gitlab_id: None,
            email: None,
            username: None,
            avatar_url: None,
            credits: Decimal::ZERO,
            role: UserRole::User,
            is_active: true,
            api_key: "test".to_string(),
            oauth_account_created_at: None,
            created_at: Utc::now(),
        };

        assert!(!user.is_admin());
        assert!(!user.is_moderator());

        user.role = UserRole::Moderator;
        assert!(!user.is_admin());
        assert!(user.is_moderator());

        user.role = UserRole::Admin;
        assert!(user.is_admin());
        assert!(user.is_moderator());
    }

    #[test]
    fn test_user_formatted_credits() {
        let mut user = User {
            id: Uuid::new_v4(),
            github_id: None,
            gitlab_id: None,
            email: None,
            username: None,
            avatar_url: None,
            credits: Decimal::from_str("10.50").unwrap(),
            role: UserRole::User,
            is_active: true,
            api_key: "test".to_string(),
            oauth_account_created_at: None,
            created_at: Utc::now(),
        };

        assert_eq!(user.formatted_credits(), "10.5");

        user.credits = Decimal::from_str("10").unwrap();
        assert_eq!(user.formatted_credits(), "10");

        user.credits = Decimal::from_str("0.010").unwrap();
        assert_eq!(user.formatted_credits(), "0.01");
    }

    #[test]
    fn test_pagination_metadata() {
        let meta = PaginationMetadata {
            current_page: 1,
            total_pages: 5,
            total_items: 50,
            page_size: 10,
        };

        assert!(!meta.has_previous());
        assert!(meta.has_next());

        let meta = PaginationMetadata {
            current_page: 3,
            total_pages: 5,
            total_items: 50,
            page_size: 10,
        };

        assert!(meta.has_previous());
        assert!(meta.has_next());

        let meta = PaginationMetadata {
            current_page: 5,
            total_pages: 5,
            total_items: 50,
            page_size: 10,
        };

        assert!(meta.has_previous());
        assert!(!meta.has_next());
    }
}
