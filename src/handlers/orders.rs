use crate::{
    middleware::auth::AuthenticatedUser,
    models::{
        self,
        order::{CheckoutRequest, CheckoutSummary, Order},
    },
    repositories::{CartRepository, OrderRepository, ProductRepository},
    services::OrderService,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_orders))
        .route("/checkout", post(checkout))
}

async fn checkout(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CheckoutRequest>,
) -> crate::Result<Json<models::ApiResponse<CheckoutSummary>>> {
    let service = order_service(&state);
    let summary = service.checkout(user.user_id, payload).await?;
    Ok(Json(models::ApiResponse::new(summary)))
}

async fn list_orders(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(pagination): Query<PaginationQuery>,
) -> crate::Result<Json<models::ApiResponse<Vec<Order>>>> {
    let service = order_service(&state);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 50);
    let offset = pagination.offset.unwrap_or(0).max(0);
    let orders = service.list_orders(user.user_id, limit, offset).await?;
    Ok(Json(models::ApiResponse::new(orders)))
}

fn order_service(state: &AppState) -> OrderService {
    OrderService::new(
        OrderRepository::new(state.db.clone()),
        ProductRepository::new(state.db.clone()),
        CartRepository::new(state.db.clone()),
    )
}
