use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::permission::Permission;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "store_status", rename_all = "PascalCase")]
pub enum StoreStatus {
    Active,
    Suspended,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Store {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub is_private: bool,
    pub status: StoreStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateStoreRequest {
    #[validate(length(min = 3, max = 255))]
    pub name: String,

    #[validate(length(min = 3, max = 64))]
    #[validate(custom(function = "crate::utils::validators::validate_slug"))]
    pub slug: String,

    #[validate(length(max = 1000))]
    pub description: Option<String>,

    #[validate(url)]
    pub logo_url: Option<String>,

    pub is_private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateStoreRequest {
    #[validate(length(min = 3, max = 255))]
    pub name: Option<String>,

    #[validate(length(max = 1000))]
    pub description: Option<String>,

    #[validate(url)]
    pub logo_url: Option<String>,

    pub is_private: Option<bool>,

    pub status: Option<StoreStatus>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "member_role", rename_all = "PascalCase")]
pub enum MemberRole {
    Owner,
    Admin,
    Manager,
    Staff,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StoreMember {
    pub id: Uuid,
    pub store_id: Uuid,
    pub user_id: Uuid,
    pub role: MemberRole,
    pub permissions: serde_json::Value,
    pub invited_by: Option<Uuid>,
    pub joined_at: DateTime<Utc>,
    pub is_active: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "access_level", rename_all = "PascalCase")]
pub enum AccessLevel {
    View,
    ViewAndBuy,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StoreAccessGrant {
    pub id: Uuid,
    pub store_id: Uuid,
    pub user_id: Uuid,
    pub granted_by: Uuid,
    pub access_level: AccessLevel,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteMemberRequest {
    pub user_id: Uuid,
    pub role: MemberRole,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreAnalyticsSummary {
    pub total_orders: i64,
    pub total_revenue: Decimal,
    pub average_order_value: Decimal,
    pub unique_customers: i64,
    pub timeframe_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreSalesPoint {
    pub date: NaiveDate,
    pub order_count: i64,
    pub total_revenue: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreTopProduct {
    pub product_id: Uuid,
    pub product_name: String,
    pub units_sold: i64,
    pub revenue: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreAnalyticsResponse {
    pub summary: StoreAnalyticsSummary,
    pub sales_trend: Vec<StoreSalesPoint>,
    pub top_products: Vec<StoreTopProduct>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn create_store_validation() {
        let valid = CreateStoreRequest {
            name: "Gadget Hub".to_string(),
            slug: "gadget-hub".to_string(),
            description: Some("Your favorite gadgets".to_string()),
            logo_url: Some("https://example.com/logo.png".to_string()),
            is_private: false,
        };
        assert!(valid.validate().is_ok());

        let invalid = CreateStoreRequest {
            name: "Go".to_string(),
            slug: "Invalid Slug".to_string(),
            description: None,
            logo_url: Some("not-a-url".to_string()),
            is_private: false,
        };
        assert!(invalid.validate().is_err());
    }
}
