use crate::{
    error::Result,
    models::store::{AccessLevel, StoreAccessGrant},
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AccessGrantRepository {
    pool: PgPool,
}

impl AccessGrantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn grant(
        &self,
        store_id: Uuid,
        user_id: Uuid,
        granted_by: Uuid,
        access_level: AccessLevel,
    ) -> Result<StoreAccessGrant> {
        let grant = sqlx::query_as::<_, StoreAccessGrant>(
            r#"
            INSERT INTO store_access_grants (store_id, user_id, granted_by, access_level)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(store_id)
        .bind(user_id)
        .bind(granted_by)
        .bind(access_level)
        .fetch_one(&self.pool)
        .await?;

        Ok(grant)
    }

    pub async fn find_active(
        &self,
        store_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<StoreAccessGrant>> {
        let grant = sqlx::query_as::<_, StoreAccessGrant>(
            r#"
            SELECT * FROM store_access_grants
            WHERE store_id = $1
              AND user_id = $2
              AND is_revoked = false
              AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(store_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(grant)
    }

    pub async fn revoke(&self, store_id: Uuid, user_id: Uuid) -> Result<Option<StoreAccessGrant>> {
        let grant = sqlx::query_as::<_, StoreAccessGrant>(
            r#"
            UPDATE store_access_grants
            SET is_revoked = true,
                revoked_at = NOW()
            WHERE store_id = $1
              AND user_id = $2
              AND is_revoked = false
            RETURNING *
            "#,
        )
        .bind(store_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(grant)
    }
}
