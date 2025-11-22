use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "order_status", rename_all = "PascalCase")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "payment_status", rename_all = "PascalCase")]
pub enum PaymentStatus {
    Pending,
    Paid,
    Failed,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderGroup {
    pub id: Uuid,
    pub user_id: Uuid,
    pub group_number: String,
    pub total_amount: Decimal,
    pub payment_status: PaymentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Order {
    pub id: Uuid,
    pub order_group_id: Uuid,
    pub user_id: Uuid,
    pub store_id: Uuid,
    pub order_number: String,
    pub status: OrderStatus,
    pub subtotal: Decimal,
    pub tax: Decimal,
    pub discount: Decimal,
    pub shipping_cost: Decimal,
    pub total_amount: Decimal,
    pub shipping_address: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub subtotal: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CartItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub added_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CartItemDetail {
    pub cart_item_id: Uuid,
    pub product_id: Uuid,
    pub store_id: Uuid,
    pub store_name: String,
    pub product_name: String,
    pub unit_price: Decimal,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AddCartItemRequest {
    pub product_id: Uuid,

    #[validate(range(min = 1, max = 1000))]
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CheckoutRequest {
    #[validate(custom(function = "crate::utils::validators::validate_shipping_address"))]
    pub shipping_address: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSummary {
    pub order_group: OrderGroup,
    pub orders: Vec<Order>,
}

impl CartItemDetail {
    pub fn group_by_store(items: &[CartItemDetail]) -> HashMap<Uuid, Vec<CartItemDetail>> {
        let mut map: HashMap<Uuid, Vec<CartItemDetail>> = HashMap::new();
        for item in items {
            map.entry(item.store_id).or_default().push(item.clone());
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_cart_item_validation() {
        let req = AddCartItemRequest {
            product_id: Uuid::new_v4(),
            quantity: 2,
        };
        assert!(req.validate().is_ok());

        let invalid = AddCartItemRequest {
            product_id: Uuid::new_v4(),
            quantity: 0,
        };
        assert!(invalid.validate().is_err());
    }
}
