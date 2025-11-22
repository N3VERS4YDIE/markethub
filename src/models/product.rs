use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Product {
    pub id: Uuid,
    pub store_id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub stock_quantity: i32,
    pub category: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateProductRequest {
    pub store_id: Uuid,

    #[validate(length(min = 3, max = 100))]
    pub sku: String,

    #[validate(length(min = 3, max = 255))]
    pub name: String,

    #[validate(length(max = 2000))]
    pub description: Option<String>,

    #[validate(range(min = 0.01, max = 1000000.0))]
    pub price: f64,

    #[validate(range(min = 0, max = 1000000))]
    pub stock_quantity: i32,

    #[validate(length(max = 100))]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateProductRequest {
    #[validate(length(min = 3, max = 255))]
    pub name: Option<String>,

    #[validate(length(max = 2000))]
    pub description: Option<String>,

    #[validate(range(min = 0.01, max = 1000000.0))]
    pub price: Option<f64>,

    #[validate(range(min = 0, max = 1000000))]
    pub stock_quantity: Option<i32>,

    #[validate(length(max = 100))]
    pub category: Option<String>,

    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductFilter {
    pub store_id: Option<Uuid>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
    pub search: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_product_validation() {
        let req = CreateProductRequest {
            store_id: Uuid::new_v4(),
            sku: "ABC-123".into(),
            name: "Awesome Product".into(),
            description: None,
            price: 99.99,
            stock_quantity: 10,
            category: None,
        };
        assert!(req.validate().is_ok());

        let invalid = CreateProductRequest {
            store_id: Uuid::new_v4(),
            sku: "A".into(),
            name: "P".into(),
            description: None,
            price: -1.0,
            stock_quantity: -5,
            category: None,
        };
        assert!(invalid.validate().is_err());
    }
}
