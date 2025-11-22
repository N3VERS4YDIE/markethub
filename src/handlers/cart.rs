use axum::{
    extract::{Path, State},
    routing::{delete, post},
    Json, Router,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    middleware::auth::AuthenticatedUser,
    models::{
        self,
        order::{AddCartItemRequest, CartItem, CartItemDetail},
    },
    repositories::{CartRepository, ProductRepository},
    services::CartService,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/items", post(add_item).get(list_items))
        .route("/items/{product_id}", delete(remove_item))
}

async fn add_item(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<AddCartItemRequest>,
) -> crate::Result<Json<models::ApiResponse<CartItem>>> {
    let service = cart_service(&state);
    let item = service.add_item(user.user_id, payload).await?;
    Ok(Json(models::ApiResponse::new(item)))
}

async fn list_items(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> crate::Result<Json<models::ApiResponse<Vec<CartItemDetail>>>> {
    let service = cart_service(&state);
    let items = service.list_items(user.user_id).await?;
    Ok(Json(models::ApiResponse::new(items)))
}

async fn remove_item(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(product_id): Path<Uuid>,
) -> crate::Result<Json<models::ApiResponse<serde_json::Value>>> {
    let service = cart_service(&state);
    service.remove_item(user.user_id, product_id).await?;
    Ok(Json(models::ApiResponse::new(json!({ "removed": true }))))
}

fn cart_service(state: &AppState) -> CartService {
    CartService::new(
        CartRepository::new(state.db.clone()),
        ProductRepository::new(state.db.clone()),
    )
}
