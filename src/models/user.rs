use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub address: Option<serde_json::Value>,
    pub loyalty_points: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub loyalty_points: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for PublicUser {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            email: value.email,
            full_name: value.full_name,
            phone: value.phone,
            loyalty_points: value.loyalty_points,
            is_active: value.is_active,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RegisterUserRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8, max = 128))]
    pub password: String,

    #[validate(length(min = 3, max = 255))]
    pub full_name: String,

    #[validate(length(max = 50))]
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokenResponse {
    pub token: String,
    pub user: PublicUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileResponse {
    pub user: PublicUser,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_user_validation() {
        let valid = RegisterUserRequest {
            email: "alice@example.com".to_string(),
            password: "verysecurepassword".to_string(),
            full_name: "Alice Example".to_string(),
            phone: Some("+1234567890".to_string()),
        };
        assert!(valid.validate().is_ok());

        let invalid = RegisterUserRequest {
            email: "invalid".to_string(),
            password: "short".to_string(),
            full_name: "Al".to_string(),
            phone: None,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn login_validation() {
        let valid = LoginRequest {
            email: "bob@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid.validate().is_ok());

        let invalid = LoginRequest {
            email: "bad".to_string(),
            password: "short".to_string(),
        };
        assert!(invalid.validate().is_err());
    }
}
