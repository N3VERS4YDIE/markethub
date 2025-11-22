use crate::{
    error::AppError,
    models::{permission::Permission, store::AccessLevel},
    repositories::{AccessGrantRepository, MemberRepository, StoreRepository},
};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PermissionService {
    stores: StoreRepository,
    members: MemberRepository,
    access_grants: AccessGrantRepository,
}

impl PermissionService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            stores: StoreRepository::new(pool.clone()),
            members: MemberRepository::new(pool.clone()),
            access_grants: AccessGrantRepository::new(pool),
        }
    }

    pub async fn ensure_store_permission(
        &self,
        user_id: Uuid,
        store_id: Uuid,
        permission: Permission,
    ) -> crate::Result<()> {
        let store = self
            .stores
            .find_by_id(store_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Store not found".into()))?;

        if self
            .members
            .find_membership(store_id, user_id)
            .await?
            .map(|member| self.member_has_permission(&member.permissions, member.role, permission))
            .unwrap_or(false)
        {
            return Ok(());
        }

        if !store.is_private
            && matches!(
                permission,
                Permission::ViewProducts | Permission::ViewOrders
            )
        {
            return Ok(());
        }

        if self
            .access_grants
            .find_active(store_id, user_id)
            .await?
            .map(|grant| access_allows(grant.access_level, permission))
            .unwrap_or(false)
        {
            return Ok(());
        }

        Err(AppError::Authorization("Insufficient permissions".into()))
    }

    fn member_has_permission(
        &self,
        permissions: &Value,
        role: crate::models::store::MemberRole,
        permission: Permission,
    ) -> bool {
        use crate::models::store::MemberRole;
        if matches!(role, MemberRole::Owner | MemberRole::Admin) {
            return true;
        }
        if let Some(list) = permissions.as_array() {
            return list
                .iter()
                .filter_map(|v| v.as_str())
                .any(|value| value.eq_ignore_ascii_case(permission.as_str()));
        }
        false
    }
}

fn access_allows(level: AccessLevel, permission: Permission) -> bool {
    match level {
        AccessLevel::View => matches!(
            permission,
            Permission::ViewProducts | Permission::ViewOrders
        ),
        AccessLevel::ViewAndBuy => matches!(
            permission,
            Permission::ViewProducts | Permission::ViewOrders | Permission::ProcessOrders
        ),
    }
}
