mod common;

use markethub::{
    error::AppError,
    models::store::{CreateStoreRequest, MemberRole},
    repositories::{MemberRepository, StoreRepository},
    services::store_service::StoreService,
};
use sqlx::PgPool;

fn store_service(pool: &PgPool) -> StoreService {
    StoreService::new(
        StoreRepository::new(pool.clone()),
        MemberRepository::new(pool.clone()),
    )
}

#[sqlx::test(migrations = "./migrations")]
async fn creating_store_bootstraps_owner_membership(pool: PgPool) {
    let owner = common::insert_user(&pool, "stores-owner@markethub.dev").await;
    let service = store_service(&pool);

    let payload = CreateStoreRequest {
        name: "Gadget Hub".into(),
        slug: "gadget-hub".into(),
        description: Some("Best gadgets".into()),
        logo_url: Some("https://example.com/logo.png".into()),
        is_private: false,
    };

    let store = service
        .create_store(owner.id, payload.clone())
        .await
        .unwrap();
    assert_eq!(store.slug, "gadget-hub");

    let members = MemberRepository::new(pool.clone());
    let membership = members
        .find_membership(store.id, owner.id)
        .await
        .unwrap()
        .expect("owner should be registered as a member");
    assert_eq!(membership.role, MemberRole::Owner);

    let err = service.create_store(owner.id, payload).await.unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));
}

#[sqlx::test(migrations = "./migrations")]
async fn list_public_returns_only_public_active_stores(pool: PgPool) {
    let owner = common::insert_user(&pool, "stores-owner2@markethub.dev").await;
    let service = store_service(&pool);

    service
        .create_store(
            owner.id,
            CreateStoreRequest {
                name: "Public Store".into(),
                slug: "public-store".into(),
                description: None,
                logo_url: None,
                is_private: false,
            },
        )
        .await
        .unwrap();

    service
        .create_store(
            owner.id,
            CreateStoreRequest {
                name: "Private Store".into(),
                slug: "private-store".into(),
                description: None,
                logo_url: None,
                is_private: true,
            },
        )
        .await
        .unwrap();

    let stores = service.list_public(10, 0).await.unwrap();
    let slugs: Vec<_> = stores.iter().map(|s| s.slug.as_str()).collect();
    assert!(slugs.contains(&"public-store"));
    assert!(!slugs.contains(&"private-store"));
}
