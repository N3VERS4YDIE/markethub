use crate::{
    error::Result,
    models::store::{CreateStoreRequest, Store, StoreStatus},
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct StoreRepository {
    pool: PgPool,
}

impl StoreRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, owner_id: Uuid, payload: &CreateStoreRequest) -> Result<Store> {
        let store = sqlx::query_as::<_, Store>(
            r#"
            INSERT INTO stores (owner_id, name, slug, description, logo_url, is_private)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(owner_id)
        .bind(&payload.name)
        .bind(&payload.slug)
        .bind(&payload.description)
        .bind(&payload.logo_url)
        .bind(payload.is_private)
        .fetch_one(&self.pool)
        .await?;

        Ok(store)
    }

    pub async fn list_public(&self, limit: i64, offset: i64) -> Result<Vec<Store>> {
        let stores = sqlx::query_as::<_, Store>(
            r#"
            SELECT * FROM stores
            WHERE is_private = false AND status = 'Active'
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(stores)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Store>> {
        let store = sqlx::query_as::<_, Store>("SELECT * FROM stores WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(store)
    }

    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<Store>> {
        let store = sqlx::query_as::<_, Store>("SELECT * FROM stores WHERE slug = $1")
            .bind(slug)
            .fetch_optional(&self.pool)
            .await?;

        Ok(store)
    }

    pub async fn update_status(&self, store_id: Uuid, status: StoreStatus) -> Result<Store> {
        let store = sqlx::query_as::<_, Store>(
            r#"
            UPDATE stores SET status = $2 WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(store_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        Ok(store)
    }
}
