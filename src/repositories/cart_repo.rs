use crate::{
    error::Result,
    models::order::{CartItem, CartItemDetail},
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct CartRepository {
    pool: PgPool,
}

impl CartRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_item(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        quantity: i32,
    ) -> Result<CartItem> {
        let item = sqlx::query_as::<_, CartItem>(
            r#"
            INSERT INTO cart_items (user_id, product_id, quantity)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, product_id)
            DO UPDATE SET quantity = cart_items.quantity + EXCLUDED.quantity,
                         updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(product_id)
        .bind(quantity)
        .fetch_one(&self.pool)
        .await?;

        Ok(item)
    }

    pub async fn update_quantity(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        quantity: i32,
    ) -> Result<CartItem> {
        let item = sqlx::query_as::<_, CartItem>(
            r#"
            UPDATE cart_items SET quantity = $3
            WHERE user_id = $1 AND product_id = $2
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(product_id)
        .bind(quantity)
        .fetch_one(&self.pool)
        .await?;

        Ok(item)
    }

    pub async fn remove_item(&self, user_id: Uuid, product_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM cart_items WHERE user_id = $1 AND product_id = $2")
            .bind(user_id)
            .bind(product_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn list_with_products(&self, user_id: Uuid) -> Result<Vec<CartItemDetail>> {
        let items = sqlx::query_as::<_, CartItemDetail>(
            r#"
            SELECT
                c.id as cart_item_id,
                c.product_id,
                p.store_id,
                s.name as store_name,
                p.name as product_name,
                p.price as unit_price,
                c.quantity
            FROM cart_items c
            JOIN products p ON p.id = c.product_id
            JOIN stores s ON s.id = p.store_id
            WHERE c.user_id = $1
            ORDER BY c.added_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    pub async fn clear_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM cart_items WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
