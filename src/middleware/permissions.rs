use uuid::Uuid;

use crate::{
    error::Result, models::permission::Permission, services::permission_service::PermissionService,
    state::AppState,
};

pub async fn ensure_store_permission(
    state: &AppState,
    user_id: Uuid,
    store_id: Uuid,
    permission: Permission,
) -> Result<()> {
    let service = PermissionService::new(state.db.clone());
    service
        .ensure_store_permission(user_id, store_id, permission)
        .await
}
