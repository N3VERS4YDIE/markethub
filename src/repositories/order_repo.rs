use crate::error::Result;
use crate::models::order::{Order, OrderGroup, OrderItem, OrderStatus, PaymentStatus};
use rust_decimal::Decimal;
use serde_json::Value;
use sqlx::{postgres::PgQueryResult, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Clone)]
pub struct OrderRepository {
    pool: PgPool,
}

impl OrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn create_group(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        group_number: &str,
        total_amount: Decimal,
        payment_status: PaymentStatus,
    ) -> Result<OrderGroup> {
        let group = sqlx::query_as::<_, OrderGroup>(
            r#"
            INSERT INTO order_groups (user_id, group_number, total_amount, payment_status)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(group_number)
        .bind(total_amount)
        .bind(payment_status)
        .fetch_one(&mut **tx)
        .await?;

        Ok(group)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_order(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        order_group_id: Uuid,
        user_id: Uuid,
        store_id: Uuid,
        order_number: &str,
        subtotal: Decimal,
        tax: Decimal,
        discount: Decimal,
        shipping_cost: Decimal,
        total_amount: Decimal,
        shipping_address: &Value,
    ) -> Result<Order> {
        let order = sqlx::query_as::<_, Order>(
            r#"
            INSERT INTO orders (
                order_group_id, user_id, store_id, order_number,
                subtotal, tax, discount, shipping_cost, total_amount, shipping_address
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8, $9, $10
            )
            RETURNING *
            "#,
        )
        .bind(order_group_id)
        .bind(user_id)
        .bind(store_id)
        .bind(order_number)
        .bind(subtotal)
        .bind(tax)
        .bind(discount)
        .bind(shipping_cost)
        .bind(total_amount)
        .bind(shipping_address)
        .fetch_one(&mut **tx)
        .await?;

        Ok(order)
    }

    pub async fn create_order_item(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        order_id: Uuid,
        product_id: Uuid,
        quantity: i32,
        unit_price: Decimal,
        subtotal: Decimal,
    ) -> Result<OrderItem> {
        let item = sqlx::query_as::<_, OrderItem>(
            r#"
            INSERT INTO order_items (order_id, product_id, quantity, unit_price, subtotal)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(order_id)
        .bind(product_id)
        .bind(quantity)
        .bind(unit_price)
        .bind(subtotal)
        .fetch_one(&mut **tx)
        .await?;

        Ok(item)
    }

    pub async fn list_orders_for_user(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Order>> {
        let orders = sqlx::query_as::<_, Order>(
            r#"
            SELECT * FROM orders
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(orders)
    }

    pub async fn update_status(&self, order_id: Uuid, status: OrderStatus) -> Result<Order> {
        let order =
            sqlx::query_as::<_, Order>("UPDATE orders SET status = $2 WHERE id = $1 RETURNING *")
                .bind(order_id)
                .bind(status)
                .fetch_one(&self.pool)
                .await?;

        Ok(order)
    }

    pub async fn mark_payment_status(
        &self,
        order_group_id: Uuid,
        status: PaymentStatus,
    ) -> Result<PgQueryResult> {
        let res = sqlx::query("UPDATE order_groups SET payment_status = $2 WHERE id = $1")
            .bind(order_group_id)
            .bind(status)
            .execute(&self.pool)
            .await?;

        Ok(res)
    }
}
