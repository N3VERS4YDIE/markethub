use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use uuid::Uuid;

use crate::{
    middleware::{auth::AuthenticatedUser, permissions::ensure_store_permission},
    models::{
        self,
        permission::Permission,
        store::{InviteMemberRequest, StoreAccessGrant},
    },
    repositories::{AccessGrantRepository, MemberRepository},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{store_id}/invite", post(invite_member))
        .route("/{store_id}/grant", post(grant_access))
        .route("/{store_id}/revoke/{user_id}", post(revoke_access))
}

async fn invite_member(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(store_id): Path<Uuid>,
    Json(payload): Json<InviteMemberRequest>,
) -> crate::Result<Json<models::ApiResponse<crate::models::store::StoreMember>>> {
    ensure_store_permission(&state, user.user_id, store_id, Permission::InviteMembers).await?;
    let repo = MemberRepository::new(state.db.clone());
    let member = repo
        .add_member(
            store_id,
            payload.user_id,
            payload.role,
            &payload.permissions,
            Some(user.user_id),
        )
        .await?;
    Ok(Json(models::ApiResponse::new(member)))
}

#[derive(Debug, serde::Deserialize)]
struct GrantAccessRequest {
    user_id: Uuid,
    #[serde(default = "default_access_level")]
    access_level: crate::models::store::AccessLevel,
}

fn default_access_level() -> crate::models::store::AccessLevel {
    crate::models::store::AccessLevel::ViewAndBuy
}

async fn grant_access(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(store_id): Path<Uuid>,
    Json(payload): Json<GrantAccessRequest>,
) -> crate::Result<Json<models::ApiResponse<StoreAccessGrant>>> {
    ensure_store_permission(&state, user.user_id, store_id, Permission::GrantAccess).await?;
    let repo = AccessGrantRepository::new(state.db.clone());
    let grant = repo
        .grant(
            store_id,
            payload.user_id,
            user.user_id,
            payload.access_level,
        )
        .await?;
    Ok(Json(models::ApiResponse::new(grant)))
}

async fn revoke_access(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path((store_id, revoke_user_id)): Path<(Uuid, Uuid)>,
) -> crate::Result<Json<models::ApiResponse<StoreAccessGrant>>> {
    ensure_store_permission(&state, user.user_id, store_id, Permission::RevokeAccess).await?;
    let repo = AccessGrantRepository::new(state.db.clone());
    let grant = repo
        .revoke(store_id, revoke_user_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Grant not found".into()))?;
    Ok(Json(models::ApiResponse::new(grant)))
}
