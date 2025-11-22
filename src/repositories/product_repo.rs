use crate::{
    error::{AppError, Result},
    models::product::Product,
};
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Clone)]
pub struct ProductRepository {
    pool: PgPool,
}

impl ProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        store_id: Uuid,
        sku: &str,
        name: &str,
        description: Option<&str>,
        price: Decimal,
        stock_quantity: i32,
        category: Option<&str>,
    ) -> Result<Product> {
        let product = sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO products (
                store_id, sku, name, description, price, stock_quantity, category
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(store_id)
        .bind(sku)
        .bind(name)
        .bind(description)
        .bind(price)
        .bind(stock_quantity)
        .bind(category)
        .fetch_one(&self.pool)
        .await?;

        Ok(product)
    }

    pub async fn find_by_id(&self, product_id: Uuid) -> Result<Option<Product>> {
        let product = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
            .bind(product_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(product)
    }

    pub async fn list_by_store(
        &self,
        store_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Product>> {
        let items = sqlx::query_as::<_, Product>(
            r#"
            SELECT * FROM products
            WHERE store_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(store_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    pub async fn update_stock(&self, product_id: Uuid, new_stock: i32) -> Result<Product> {
        let product = sqlx::query_as::<_, Product>(
            r#"
            UPDATE products SET stock_quantity = $2
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(product_id)
        .bind(new_stock)
        .fetch_one(&self.pool)
        .await?;

        Ok(product)
    }

    pub async fn save(&self, product: &Product) -> Result<Product> {
        let updated = sqlx::query_as::<_, Product>(
            r#"
            UPDATE products
            SET name = $2,
                description = $3,
                price = $4,
                stock_quantity = $5,
                category = $6,
                is_active = $7
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(product.id)
        .bind(&product.name)
        .bind(&product.description)
        .bind(product.price)
        .bind(product.stock_quantity)
        .bind(&product.category)
        .bind(product.is_active)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    pub async fn decrement_stock(&self, product_id: Uuid, qty: i32) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE products SET stock_quantity = stock_quantity - $2
            WHERE id = $1 AND stock_quantity >= $2
            "#,
        )
        .bind(product_id)
        .bind(qty)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::Conflict("Insufficient stock".into()));
        }

        Ok(())
    }

    pub async fn decrement_stock_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        product_id: Uuid,
        qty: i32,
    ) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE products SET stock_quantity = stock_quantity - $2
            WHERE id = $1 AND stock_quantity >= $2
            "#,
        )
        .bind(product_id)
        .bind(qty)
        .execute(&mut **tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::Conflict("Insufficient stock".into()));
        }

        Ok(())
    }
}
