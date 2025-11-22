use crate::{
    error::Result,
    models::{
        permission::Permission,
        store::{MemberRole, StoreMember},
    },
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct MemberRepository {
    pool: PgPool,
}

impl MemberRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn add_member(
        &self,
        store_id: Uuid,
        user_id: Uuid,
        role: MemberRole,
        permissions: &[Permission],
        invited_by: Option<Uuid>,
    ) -> Result<StoreMember> {
        let permissions_json = json!(permissions.iter().map(|p| p.as_str()).collect::<Vec<_>>());

        let member = sqlx::query_as::<_, StoreMember>(
            r#"
            INSERT INTO store_members (store_id, user_id, role, permissions, invited_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(store_id)
        .bind(user_id)
        .bind(role)
        .bind(permissions_json)
        .bind(invited_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(member)
    }

    pub async fn find_membership(
        &self,
        store_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<StoreMember>> {
        let member = sqlx::query_as::<_, StoreMember>(
            r#"
            SELECT * FROM store_members
            WHERE store_id = $1 AND user_id = $2 AND is_active = true
            "#,
        )
        .bind(store_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(member)
    }

    pub async fn list_members(&self, store_id: Uuid) -> Result<Vec<StoreMember>> {
        let members = sqlx::query_as::<_, StoreMember>(
            r#"
            SELECT * FROM store_members
            WHERE store_id = $1
            ORDER BY joined_at DESC
            "#,
        )
        .bind(store_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(members)
    }
}
