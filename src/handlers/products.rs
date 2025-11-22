use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    middleware::{
        auth::{AuthenticatedUser, MaybeAuthenticatedUser},
        permissions::ensure_store_permission,
    },
    models::{
        self,
        permission::Permission,
        product::{CreateProductRequest, Product},
    },
    repositories::StoreRepository,
    services::ProductService,
    state::AppState,
};

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_product))
        .route("/store/{store_id}", get(list_store_products))
}

async fn create_product(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateProductRequest>,
) -> crate::Result<Json<models::ApiResponse<Product>>> {
    ensure_store_permission(
        &state,
        user.user_id,
        payload.store_id,
        Permission::CreateProducts,
    )
    .await?;
    let service = product_service(&state);
    let product = service.create_product(payload).await?;
    Ok(Json(models::ApiResponse::new(product)))
}

async fn list_store_products(
    State(state): State<AppState>,
    Path(store_id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
    MaybeAuthenticatedUser(maybe_user): MaybeAuthenticatedUser,
) -> crate::Result<Json<models::ApiResponse<Vec<Product>>>> {
    let store_repo = StoreRepository::new(state.db.clone());
    let store = store_repo
        .find_by_id(store_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Store not found".into()))?;

    if store.is_private {
        let user = maybe_user.ok_or_else(|| {
            crate::error::AppError::Authentication("Authentication required".into())
        })?;
        ensure_store_permission(&state, user.user_id, store_id, Permission::ViewProducts).await?;
    }

    let limit = pagination.limit.unwrap_or(20).clamp(1, 50);
    let offset = pagination.offset.unwrap_or(0).max(0);
    let service = product_service(&state);
    let products = service.list_by_store(store_id, limit, offset).await?;
    Ok(Json(models::ApiResponse::new(products)))
}

fn product_service(state: &AppState) -> ProductService {
    ProductService::new(
        crate::repositories::ProductRepository::new(state.db.clone()),
        StoreRepository::new(state.db.clone()),
    )
}
