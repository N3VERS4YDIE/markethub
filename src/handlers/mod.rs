use axum::{extract::State, routing::get, Json, Router};
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

pub mod auth;
pub mod cart;
pub mod members;
pub mod orders;
pub mod products;
pub mod stores;
pub mod users;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .nest("/api/v1/auth", auth::router())
        .nest("/api/v1/users", users::router())
        .nest("/api/v1/stores", stores::router())
        .nest("/api/v1/products", products::router())
        .nest("/api/v1/cart", cart::router())
        .nest("/api/v1/orders", orders::router())
        .nest("/api/v1/members", members::router())
}

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "markethub",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn metrics(State(state): State<AppState>) -> Result<String, AppError> {
    state
        .metrics
        .encode()
        .map_err(|err| AppError::Internal(err.into()))
}
