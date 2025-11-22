mod common;

use markethub::{
    error::AppError,
    models::{permission::Permission, store::AccessLevel},
    repositories::AccessGrantRepository,
    services::permission_service::PermissionService,
};
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn owners_have_full_permissions(pool: PgPool) {
    let owner = common::insert_user(&pool, "owner@markethub.dev").await;
    let store = common::create_store(&pool, owner.id, "owner-store", false).await;

    let service = PermissionService::new(pool.clone());
    service
        .ensure_store_permission(owner.id, store.id, Permission::EditPermissions)
        .await
        .expect("owners should have every permission");
}

#[sqlx::test(migrations = "./migrations")]
async fn access_grants_enforce_view_only_levels(pool: PgPool) {
    let owner = common::insert_user(&pool, "owner2@markethub.dev").await;
    let viewer = common::insert_user(&pool, "viewer@markethub.dev").await;
    let store = common::create_store(&pool, owner.id, "private-store", true).await;

    let service = PermissionService::new(pool.clone());
    let err = service
        .ensure_store_permission(viewer.id, store.id, Permission::ViewProducts)
        .await
        .expect_err("viewer should not have access yet");
    assert!(matches!(err, AppError::Authorization(_)));

    let grants = AccessGrantRepository::new(pool.clone());
    grants
        .grant(store.id, viewer.id, owner.id, AccessLevel::View)
        .await
        .unwrap();

    service
        .ensure_store_permission(viewer.id, store.id, Permission::ViewProducts)
        .await
        .expect("grant should allow viewing products");

    let err = service
        .ensure_store_permission(viewer.id, store.id, Permission::ProcessOrders)
        .await
        .expect_err("view-only grants should not allow order processing");
    assert!(matches!(err, AppError::Authorization(_)));
}

#[sqlx::test(migrations = "./migrations")]
async fn public_stores_allow_guest_viewing(pool: PgPool) {
    let owner = common::insert_user(&pool, "owner3@markethub.dev").await;
    let guest = common::insert_user(&pool, "guest@markethub.dev").await;
    let store = common::create_store(&pool, owner.id, "public-store", false).await;

    let service = PermissionService::new(pool.clone());
    service
        .ensure_store_permission(guest.id, store.id, Permission::ViewProducts)
        .await
        .expect("public stores should allow viewing products without membership");
}
