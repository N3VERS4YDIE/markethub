#![allow(dead_code)]

use std::sync::Arc;

use markethub::{
    metrics::Metrics,
    models::{
        product::{CreateProductRequest, Product},
        store::{CreateStoreRequest, Store},
        user::User,
    },
    repositories::{MemberRepository, ProductRepository, StoreRepository},
    services::{ProductService, StoreService},
    state::AppState,
    utils::{jwt::JwtConfig, password},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub fn test_jwt() -> Arc<JwtConfig> {
    Arc::new(JwtConfig::new("test-secret", 24))
}

pub fn build_state(pool: PgPool) -> AppState {
    AppState::new(
        pool,
        JwtConfig::new("test-secret", 24),
        Arc::new(Metrics::default()),
    )
}

pub async fn insert_user(pool: &PgPool, email: &str) -> User {
    let hash = password::hash_password("SuperSecure123!").expect("hashing should work");

    sqlx::query_as::<_, User>(
        r#"
		INSERT INTO users (email, password_hash, full_name, phone)
		VALUES ($1, $2, $3, $4)
		RETURNING *
		"#,
    )
    .bind(email)
    .bind(&hash)
    .bind("Test User")
    .bind(Some("+1234567890"))
    .fetch_one(pool)
    .await
    .expect("user insert should succeed")
}

pub async fn create_store(pool: &PgPool, owner_id: Uuid, slug: &str, is_private: bool) -> Store {
    let stores = StoreRepository::new(pool.clone());
    let members = MemberRepository::new(pool.clone());
    let service = StoreService::new(stores, members);

    service
        .create_store(
            owner_id,
            CreateStoreRequest {
                name: format!("{} Store", slug.replace('-', " ")),
                slug: slug.to_string(),
                description: Some("A test store".into()),
                logo_url: Some("https://example.com/logo.png".into()),
                is_private,
            },
        )
        .await
        .expect("store creation should succeed")
}

pub async fn create_product(
    pool: &PgPool,
    store_id: Uuid,
    sku: &str,
    price: f64,
    stock: i32,
) -> Product {
    let products = ProductRepository::new(pool.clone());
    let stores = StoreRepository::new(pool.clone());
    let service = ProductService::new(products, stores);

    service
        .create_product(CreateProductRequest {
            store_id,
            sku: sku.to_string(),
            name: format!("Product {}", sku),
            description: Some("Test product".into()),
            price,
            stock_quantity: stock,
            category: None,
        })
        .await
        .expect("product creation should succeed")
}

pub fn shipping_address() -> Value {
    json!({
        "line1": "123 Test St",
        "city": "Testville",
        "country": "US"
    })
}
