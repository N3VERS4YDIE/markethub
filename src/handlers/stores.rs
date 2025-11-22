use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    middleware::{auth::AuthenticatedUser, permissions::ensure_store_permission},
    models::{
        self,
        permission::Permission,
        store::{CreateStoreRequest, Store, StoreAnalyticsResponse, StoreMember},
    },
    repositories::{AnalyticsRepository, MemberRepository, StoreRepository},
    services::{AnalyticsService, StoreService},
    state::AppState,
};

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct AnalyticsQuery {
    days: Option<i64>,
    top: Option<i64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_store).get(list_stores))
        .route("/{store_id}/members", get(list_members))
        .route("/{store_id}/analytics", get(store_analytics))
}

async fn create_store(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateStoreRequest>,
) -> crate::Result<Json<models::ApiResponse<Store>>> {
    let service = store_service(&state);
    let store = service.create_store(user.user_id, payload).await?;
    Ok(Json(models::ApiResponse::new(store)))
}

async fn list_stores(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> crate::Result<Json<models::ApiResponse<Vec<Store>>>> {
    let service = store_service(&state);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 50);
    let offset = pagination.offset.unwrap_or(0).max(0);
    let stores = service.list_public(limit, offset).await?;
    Ok(Json(models::ApiResponse::new(stores)))
}

async fn list_members(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(store_id): Path<Uuid>,
) -> crate::Result<Json<models::ApiResponse<Vec<StoreMember>>>> {
    ensure_store_permission(&state, user.user_id, store_id, Permission::ViewMembers).await?;
    let service = store_service(&state);
    let members = service.list_members(store_id).await?;
    Ok(Json(models::ApiResponse::new(members)))
}

async fn store_analytics(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(store_id): Path<Uuid>,
    Query(query): Query<AnalyticsQuery>,
) -> crate::Result<Json<models::ApiResponse<StoreAnalyticsResponse>>> {
    ensure_store_permission(&state, user.user_id, store_id, Permission::ViewStats).await?;

    let days = query.days.unwrap_or(30).clamp(1, 180);
    let top = query.top.unwrap_or(5).clamp(1, 50);

    let service = analytics_service(&state);
    let analytics = service.store_analytics(store_id, days, top).await?;

    Ok(Json(models::ApiResponse::new(analytics)))
}

fn store_service(state: &AppState) -> StoreService {
    StoreService::new(
        StoreRepository::new(state.db.clone()),
        MemberRepository::new(state.db.clone()),
    )
}

fn analytics_service(state: &AppState) -> AnalyticsService {
    AnalyticsService::new(
        StoreRepository::new(state.db.clone()),
        AnalyticsRepository::new(state.db.clone()),
    )
}
