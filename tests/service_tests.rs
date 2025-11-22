// Service layer tests for auth, cart, order, and permission services
mod common;

use common::{create_product, create_store, test_jwt};
use markethub::{
    models::{
        order::AddCartItemRequest,
        user::{LoginRequest, RegisterUserRequest},
    },
    repositories::{CartRepository, ProductRepository, UserRepository},
    services::{auth_service::AuthService, cart_service::CartService, user_service::UserService},
};
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_test_db() -> PgPool {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://markethub:password@localhost:5432/markethub".to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

// ========== AUTH SERVICE TESTS ==========

#[tokio::test]
async fn auth_register_success() {
    let pool = setup_test_db().await;
    let user_repo = UserRepository::new(pool.clone());
    let jwt_config = test_jwt();
    let service = AuthService::new(user_repo, jwt_config);

    let request = RegisterUserRequest {
        email: format!("newuser-{}@test.com", Uuid::new_v4()),
        password: "SecurePass123!".to_string(),
        full_name: "New User".to_string(),
        phone: Some("+1234567890".to_string()),
    };

    let result = service.register(request.clone()).await;
    assert!(result.is_ok(), "Registration should succeed");

    let response = result.unwrap();
    assert!(!response.token.is_empty(), "Token should be generated");
    assert_eq!(response.user.email, request.email);
    assert_eq!(response.user.full_name, "New User");
}

#[tokio::test]
async fn auth_register_duplicate_email() {
    let pool = setup_test_db().await;
    let user_repo = UserRepository::new(pool.clone());
    let jwt_config = test_jwt();
    let service = AuthService::new(user_repo, jwt_config);

    let email = format!("duplicate-{}@test.com", Uuid::new_v4());
    let request = RegisterUserRequest {
        email: email.clone(),
        password: "SecurePass123!".to_string(),
        full_name: "First User".to_string(),
        phone: None,
    };

    service.register(request.clone()).await.unwrap();
    let second = service.register(request).await;

    assert!(second.is_err(), "Should fail with duplicate email");
    assert!(matches!(
        second.unwrap_err(),
        markethub::error::AppError::Conflict(_)
    ));
}

#[tokio::test]
async fn auth_register_invalid_email() {
    let pool = setup_test_db().await;
    let user_repo = UserRepository::new(pool.clone());
    let jwt_config = test_jwt();
    let service = AuthService::new(user_repo, jwt_config);

    let request = RegisterUserRequest {
        email: "not-an-email".to_string(),
        password: "SecurePass123!".to_string(),
        full_name: "Test User".to_string(),
        phone: None,
    };

    let result = service.register(request).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        markethub::error::AppError::Validation(_)
    ));
}

#[tokio::test]
async fn auth_login_success() {
    let pool = setup_test_db().await;
    let user_repo = UserRepository::new(pool.clone());
    let jwt_config = test_jwt();
    let service = AuthService::new(user_repo, jwt_config);

    let email = format!("login-{}@test.com", Uuid::new_v4());
    let password = "SecurePass123!".to_string();

    let register_req = RegisterUserRequest {
        email: email.clone(),
        password: password.clone(),
        full_name: "Login User".to_string(),
        phone: None,
    };
    service.register(register_req).await.unwrap();

    let login_req = LoginRequest {
        email: email.clone(),
        password,
    };
    let result = service.login(login_req).await;

    assert!(result.is_ok(), "Login should succeed");
    let response = result.unwrap();
    assert!(!response.token.is_empty());
    assert_eq!(response.user.email, email);
}

#[tokio::test]
async fn auth_login_wrong_password() {
    let pool = setup_test_db().await;
    let user_repo = UserRepository::new(pool.clone());
    let jwt_config = test_jwt();
    let service = AuthService::new(user_repo, jwt_config);

    let email = format!("wrongpass-{}@test.com", Uuid::new_v4());
    service
        .register(RegisterUserRequest {
            email: email.clone(),
            password: "SecurePass123!".to_string(),
            full_name: "Test User".to_string(),
            phone: None,
        })
        .await
        .unwrap();

    let login_req = LoginRequest {
        email,
        password: "WrongPassword!".to_string(),
    };

    let result = service.login(login_req).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        markethub::error::AppError::Authentication(_)
    ));
}

// ========== CART SERVICE TESTS ==========

async fn create_test_user(pool: &PgPool, email: &str) -> Uuid {
    let user_repo = UserRepository::new(pool.clone());
    let jwt_config = test_jwt();
    let auth_service = AuthService::new(user_repo, jwt_config);

    auth_service
        .register(RegisterUserRequest {
            email: email.to_string(),
            password: "Pass123!".to_string(),
            full_name: "User".to_string(),
            phone: None,
        })
        .await
        .unwrap()
        .user
        .id
}

async fn create_test_setup(pool: &PgPool, email_prefix: &str) -> (Uuid, Uuid, Uuid, CartService) {
    let user_id = create_test_user(
        pool,
        &format!("{}-user-{}@test.com", email_prefix, Uuid::new_v4()),
    )
    .await;
    let owner_id = create_test_user(
        pool,
        &format!("{}-owner-{}@test.com", email_prefix, Uuid::new_v4()),
    )
    .await;

    let slug = format!("store-{}", Uuid::new_v4());
    let store = create_store(pool, owner_id, &slug, false).await;

    let sku = format!("SKU-{}", Uuid::new_v4());
    let product = create_product(pool, store.id, &sku, 100.0, 10).await;

    let cart_repo = CartRepository::new(pool.clone());
    let product_repo = ProductRepository::new(pool.clone());
    let cart_service = CartService::new(cart_repo, product_repo);

    (user_id, store.id, product.id, cart_service)
}

#[tokio::test]
async fn cart_add_item_success() {
    let pool = setup_test_db().await;
    let (user_id, _, product_id, service) = create_test_setup(&pool, "cart1").await;

    let result = service
        .add_item(
            user_id,
            AddCartItemRequest {
                product_id,
                quantity: 2,
            },
        )
        .await;

    assert!(result.is_ok());
    let item = result.unwrap();
    assert_eq!(item.quantity, 2);
}

#[tokio::test]
async fn cart_add_item_insufficient_stock() {
    let pool = setup_test_db().await;
    let (user_id, _, product_id, service) = create_test_setup(&pool, "cart2").await;

    let result = service
        .add_item(
            user_id,
            AddCartItemRequest {
                product_id,
                quantity: 999,
            },
        )
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        markethub::error::AppError::Conflict(_)
    ));
}

#[tokio::test]
async fn cart_remove_item() {
    let pool = setup_test_db().await;
    let (user_id, _, product_id, service) = create_test_setup(&pool, "cart3").await;

    service
        .add_item(
            user_id,
            AddCartItemRequest {
                product_id,
                quantity: 2,
            },
        )
        .await
        .unwrap();

    let result = service.remove_item(user_id, product_id).await;
    assert!(result.is_ok());

    let items = service.list_items(user_id).await.unwrap();
    assert_eq!(items.len(), 0);
}

#[tokio::test]
async fn cart_clear() {
    let pool = setup_test_db().await;
    let (user_id, _, product_id, service) = create_test_setup(&pool, "cart4").await;

    service
        .add_item(
            user_id,
            AddCartItemRequest {
                product_id,
                quantity: 2,
            },
        )
        .await
        .unwrap();

    let result = service.clear(user_id).await;
    assert!(result.is_ok());

    let items = service.list_items(user_id).await.unwrap();
    assert_eq!(items.len(), 0);
}

// ========== ORDER SERVICE TESTS ==========

use markethub::{
    models::order::CheckoutRequest, repositories::OrderRepository,
    services::order_service::OrderService,
};
use serde_json::json;

#[tokio::test]
async fn order_checkout_success() {
    let pool = setup_test_db().await;
    let (user_id, _, product_id, cart_service) = create_test_setup(&pool, "order1").await;

    // Add item to cart
    cart_service
        .add_item(
            user_id,
            AddCartItemRequest {
                product_id,
                quantity: 2,
            },
        )
        .await
        .unwrap();

    // Create order service
    let order_repo = OrderRepository::new(pool.clone());
    let product_repo = ProductRepository::new(pool.clone());
    let cart_repo = CartRepository::new(pool.clone());
    let order_service = OrderService::new(order_repo, product_repo, cart_repo);

    // Checkout
    let result = order_service
        .checkout(
            user_id,
            CheckoutRequest {
                shipping_address: json!({"street": "123 Main St", "city": "Test City"}),
            },
        )
        .await;

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert_eq!(summary.orders.len(), 1);
    assert!(summary.order_group.total_amount > rust_decimal::Decimal::ZERO);

    // Verify cart is cleared
    let items = cart_service.list_items(user_id).await.unwrap();
    assert_eq!(items.len(), 0);
}

#[tokio::test]
async fn order_checkout_empty_cart() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool, &format!("order2-{}@test.com", Uuid::new_v4())).await;

    let order_repo = OrderRepository::new(pool.clone());
    let product_repo = ProductRepository::new(pool.clone());
    let cart_repo = CartRepository::new(pool.clone());
    let order_service = OrderService::new(order_repo, product_repo, cart_repo);

    let result = order_service
        .checkout(
            user_id,
            CheckoutRequest {
                shipping_address: json!({"street": "123 Main St"}),
            },
        )
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        markethub::error::AppError::BadRequest(_)
    ));
}

#[tokio::test]
async fn order_checkout_multi_store() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool, &format!("order3-{}@test.com", Uuid::new_v4())).await;
    let owner1_id = create_test_user(&pool, &format!("owner3a-{}@test.com", Uuid::new_v4())).await;
    let owner2_id = create_test_user(&pool, &format!("owner3b-{}@test.com", Uuid::new_v4())).await;

    // Create two stores
    let store1 = create_store(
        &pool,
        owner1_id,
        &format!("store-{}", Uuid::new_v4()),
        false,
    )
    .await;
    let store2 = create_store(
        &pool,
        owner2_id,
        &format!("store-{}", Uuid::new_v4()),
        false,
    )
    .await;

    // Create products
    let product1 = create_product(
        &pool,
        store1.id,
        &format!("SKU-{}", Uuid::new_v4()),
        100.0,
        10,
    )
    .await;
    let product2 = create_product(
        &pool,
        store2.id,
        &format!("SKU-{}", Uuid::new_v4()),
        200.0,
        10,
    )
    .await;

    // Add to cart
    let cart_repo = CartRepository::new(pool.clone());
    let product_repo = ProductRepository::new(pool.clone());
    let cart_service = CartService::new(cart_repo.clone(), product_repo.clone());

    cart_service
        .add_item(
            user_id,
            AddCartItemRequest {
                product_id: product1.id,
                quantity: 2,
            },
        )
        .await
        .unwrap();
    cart_service
        .add_item(
            user_id,
            AddCartItemRequest {
                product_id: product2.id,
                quantity: 3,
            },
        )
        .await
        .unwrap();

    // Checkout
    let order_repo = OrderRepository::new(pool.clone());
    let order_service = OrderService::new(order_repo, product_repo, cart_repo);

    let result = order_service
        .checkout(
            user_id,
            CheckoutRequest {
                shipping_address: json!({"street": "456 Oak Ave"}),
            },
        )
        .await;

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert_eq!(
        summary.orders.len(),
        2,
        "Should create 2 orders (one per store)"
    );
}

#[tokio::test]
async fn order_checkout_decrements_stock() {
    let pool = setup_test_db().await;
    let (user_id, _, product_id, cart_service) = create_test_setup(&pool, "order4").await;

    let product_repo = ProductRepository::new(pool.clone());

    // Get initial stock
    let initial_product = product_repo.find_by_id(product_id).await.unwrap().unwrap();
    let initial_stock = initial_product.stock_quantity;

    // Add to cart
    cart_service
        .add_item(
            user_id,
            AddCartItemRequest {
                product_id,
                quantity: 3,
            },
        )
        .await
        .unwrap();

    // Checkout
    let order_repo = OrderRepository::new(pool.clone());
    let cart_repo = CartRepository::new(pool.clone());
    let order_service = OrderService::new(order_repo, product_repo.clone(), cart_repo);

    order_service
        .checkout(
            user_id,
            CheckoutRequest {
                shipping_address: json!({"street": "789 Elm St"}),
            },
        )
        .await
        .unwrap();

    // Verify stock decreased
    let updated_product = product_repo.find_by_id(product_id).await.unwrap().unwrap();
    assert_eq!(updated_product.stock_quantity, initial_stock - 3);
}

#[tokio::test]
async fn order_list_orders() {
    let pool = setup_test_db().await;
    let (user_id, _, product_id, cart_service) = create_test_setup(&pool, "order5").await;

    let order_repo = OrderRepository::new(pool.clone());
    let product_repo = ProductRepository::new(pool.clone());
    let cart_repo = CartRepository::new(pool.clone());
    let order_service = OrderService::new(order_repo, product_repo, cart_repo);

    // Create first order
    cart_service
        .add_item(
            user_id,
            AddCartItemRequest {
                product_id,
                quantity: 1,
            },
        )
        .await
        .unwrap();
    order_service
        .checkout(
            user_id,
            CheckoutRequest {
                shipping_address: json!({"street": "A St"}),
            },
        )
        .await
        .unwrap();

    // Create second order
    cart_service
        .add_item(
            user_id,
            AddCartItemRequest {
                product_id,
                quantity: 2,
            },
        )
        .await
        .unwrap();
    order_service
        .checkout(
            user_id,
            CheckoutRequest {
                shipping_address: json!({"street": "B St"}),
            },
        )
        .await
        .unwrap();

    // List orders
    let orders = order_service.list_orders(user_id, 10, 0).await.unwrap();
    assert!(orders.len() >= 2);
}

// ========== PERMISSION SERVICE TESTS ==========

use markethub::{
    models::{
        permission::Permission,
        store::{AccessLevel, MemberRole},
    },
    repositories::{AccessGrantRepository, MemberRepository, StoreRepository},
    services::permission_service::PermissionService,
};

#[tokio::test]
async fn permission_owner_has_all_permissions() {
    let pool = setup_test_db().await;
    let owner_id = create_test_user(&pool, &format!("perm1-{}@test.com", Uuid::new_v4())).await;
    let store = create_store(&pool, owner_id, &format!("store-{}", Uuid::new_v4()), false).await;

    let permission_service = PermissionService::new(pool.clone());

    let permissions = vec![
        Permission::ViewProducts,
        Permission::CreateProducts,
        Permission::EditProducts,
        Permission::DeleteProducts,
        Permission::ViewOrders,
    ];

    for permission in permissions {
        let result = permission_service
            .ensure_store_permission(owner_id, store.id, permission)
            .await;
        assert!(result.is_ok(), "Owner should have {:?}", permission);
    }
}

#[tokio::test]
async fn permission_non_member_cannot_access_private_store() {
    let pool = setup_test_db().await;
    let owner_id = create_test_user(&pool, &format!("perm2a-{}@test.com", Uuid::new_v4())).await;
    let user_id = create_test_user(&pool, &format!("perm2b-{}@test.com", Uuid::new_v4())).await;
    let store = create_store(&pool, owner_id, &format!("store-{}", Uuid::new_v4()), true).await; // Private

    let permission_service = PermissionService::new(pool.clone());

    let result = permission_service
        .ensure_store_permission(user_id, store.id, Permission::ViewProducts)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn permission_anyone_can_view_public_store() {
    let pool = setup_test_db().await;
    let owner_id = create_test_user(&pool, &format!("perm3a-{}@test.com", Uuid::new_v4())).await;
    let user_id = create_test_user(&pool, &format!("perm3b-{}@test.com", Uuid::new_v4())).await;
    let store = create_store(&pool, owner_id, &format!("store-{}", Uuid::new_v4()), false).await; // Public

    let permission_service = PermissionService::new(pool.clone());

    let result = permission_service
        .ensure_store_permission(user_id, store.id, Permission::ViewProducts)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn permission_admin_has_most_permissions() {
    let pool = setup_test_db().await;
    let owner_id = create_test_user(&pool, &format!("perm4a-{}@test.com", Uuid::new_v4())).await;
    let admin_id = create_test_user(&pool, &format!("perm4b-{}@test.com", Uuid::new_v4())).await;
    let store = create_store(&pool, owner_id, &format!("store-{}", Uuid::new_v4()), false).await;

    let member_repo = MemberRepository::new(pool.clone());
    member_repo
        .add_member(store.id, admin_id, MemberRole::Admin, &[], Some(owner_id))
        .await
        .unwrap();

    let permission_service = PermissionService::new(pool.clone());

    let permissions = vec![
        Permission::ViewProducts,
        Permission::CreateProducts,
        Permission::ViewOrders,
        Permission::ProcessOrders,
    ];

    for permission in permissions {
        let result = permission_service
            .ensure_store_permission(admin_id, store.id, permission)
            .await;
        assert!(result.is_ok(), "Admin should have {:?}", permission);
    }
}

#[tokio::test]
async fn permission_access_grant_view_level() {
    let pool = setup_test_db().await;
    let owner_id = create_test_user(&pool, &format!("perm5a-{}@test.com", Uuid::new_v4())).await;
    let guest_id = create_test_user(&pool, &format!("perm5b-{}@test.com", Uuid::new_v4())).await;
    let store = create_store(&pool, owner_id, &format!("store-{}", Uuid::new_v4()), true).await; // Private

    let access_grant_repo = AccessGrantRepository::new(pool.clone());
    access_grant_repo
        .grant(store.id, guest_id, owner_id, AccessLevel::View)
        .await
        .unwrap();

    let permission_service = PermissionService::new(pool.clone());

    // Can view
    let result = permission_service
        .ensure_store_permission(guest_id, store.id, Permission::ViewProducts)
        .await;
    assert!(result.is_ok());

    // Cannot create
    let result = permission_service
        .ensure_store_permission(guest_id, store.id, Permission::CreateProducts)
        .await;
    assert!(result.is_err());
}

// ========== STORE SERVICE TESTS ==========

use markethub::{models::store::CreateStoreRequest, services::store_service::StoreService};

#[tokio::test]
async fn store_create_success() {
    let pool = setup_test_db().await;
    let owner_id = create_test_user(&pool, &format!("store1-{}@test.com", Uuid::new_v4())).await;

    let store_repo = StoreRepository::new(pool.clone());
    let member_repo = MemberRepository::new(pool.clone());
    let service = StoreService::new(store_repo, member_repo.clone());

    let slug = format!("my-store-{}", Uuid::new_v4());
    let result = service
        .create_store(
            owner_id,
            CreateStoreRequest {
                name: "My Store".to_string(),
                slug: slug.clone(),
                description: Some("Test store".to_string()),
                logo_url: None,
                is_private: false,
            },
        )
        .await;

    assert!(result.is_ok());
    let store = result.unwrap();
    assert_eq!(store.slug, slug);
    assert_eq!(store.owner_id, owner_id);

    // Verify owner membership was created
    let members = member_repo.list_members(store.id).await.unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].user_id, owner_id);
    assert_eq!(members[0].role, MemberRole::Owner);
}

#[tokio::test]
async fn store_create_duplicate_slug() {
    let pool = setup_test_db().await;
    let owner_id = create_test_user(&pool, &format!("store2-{}@test.com", Uuid::new_v4())).await;

    let store_repo = StoreRepository::new(pool.clone());
    let member_repo = MemberRepository::new(pool.clone());
    let service = StoreService::new(store_repo, member_repo);

    let slug = format!("duplicate-{}", Uuid::new_v4());

    // Create first store
    service
        .create_store(
            owner_id,
            CreateStoreRequest {
                name: "First Store".to_string(),
                slug: slug.clone(),
                description: None,
                logo_url: None,
                is_private: false,
            },
        )
        .await
        .unwrap();

    // Try to create second store with same slug
    let result = service
        .create_store(
            owner_id,
            CreateStoreRequest {
                name: "Second Store".to_string(),
                slug: slug.clone(),
                description: None,
                logo_url: None,
                is_private: false,
            },
        )
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        markethub::error::AppError::Conflict(_)
    ));
}

#[tokio::test]
async fn store_list_public() {
    let pool = setup_test_db().await;
    let owner_id = create_test_user(&pool, &format!("store3-{}@test.com", Uuid::new_v4())).await;

    let store_repo = StoreRepository::new(pool.clone());
    let member_repo = MemberRepository::new(pool.clone());
    let service = StoreService::new(store_repo, member_repo);

    // Create public store
    create_store(
        &pool,
        owner_id,
        &format!("public-{}", Uuid::new_v4()),
        false,
    )
    .await;

    // Create private store
    create_store(
        &pool,
        owner_id,
        &format!("private-{}", Uuid::new_v4()),
        true,
    )
    .await;

    let stores = service.list_public(100, 0).await.unwrap();

    // All returned stores should be public
    for store in &stores {
        assert!(!store.is_private);
    }
}

#[tokio::test]
async fn store_get_store() {
    let pool = setup_test_db().await;
    let owner_id = create_test_user(&pool, &format!("store4-{}@test.com", Uuid::new_v4())).await;
    let store = create_store(
        &pool,
        owner_id,
        &format!("get-test-{}", Uuid::new_v4()),
        false,
    )
    .await;

    let store_repo = StoreRepository::new(pool.clone());
    let member_repo = MemberRepository::new(pool.clone());
    let service = StoreService::new(store_repo, member_repo);

    let result = service.get_store(store.id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, store.id);
}

#[tokio::test]
async fn store_list_members() {
    let pool = setup_test_db().await;
    let owner_id = create_test_user(&pool, &format!("store5-{}@test.com", Uuid::new_v4())).await;
    let store = create_store(
        &pool,
        owner_id,
        &format!("members-{}", Uuid::new_v4()),
        false,
    )
    .await;

    let store_repo = StoreRepository::new(pool.clone());
    let member_repo = MemberRepository::new(pool.clone());
    let service = StoreService::new(store_repo, member_repo);

    let members = service.list_members(store.id).await.unwrap();
    assert!(!members.is_empty()); // At least owner
    assert_eq!(members[0].user_id, owner_id);
}

// ========== PRODUCT SERVICE TESTS ==========

use markethub::{
    models::product::{CreateProductRequest, UpdateProductRequest},
    services::product_service::ProductService,
};

#[tokio::test]
async fn product_create_success() {
    let pool = setup_test_db().await;
    let owner_id = create_test_user(&pool, &format!("prod1-{}@test.com", Uuid::new_v4())).await;
    let store = create_store(&pool, owner_id, &format!("store-{}", Uuid::new_v4()), false).await;

    let product_repo = ProductRepository::new(pool.clone());
    let store_repo = StoreRepository::new(pool.clone());
    let service = ProductService::new(product_repo, store_repo);

    let sku = format!("SKU-{}", Uuid::new_v4());
    let result = service
        .create_product(CreateProductRequest {
            store_id: store.id,
            sku: sku.clone(),
            name: "Test Product".to_string(),
            description: Some("A test product".to_string()),
            price: 99.99,
            stock_quantity: 50,
            category: Some("Electronics".to_string()),
        })
        .await;

    assert!(result.is_ok());
    let product = result.unwrap();
    assert_eq!(product.sku, sku);
    assert_eq!(product.name, "Test Product");
    assert_eq!(product.stock_quantity, 50);
}

#[tokio::test]
async fn product_create_invalid_store() {
    let pool = setup_test_db().await;
    let fake_store_id = Uuid::new_v4();

    let product_repo = ProductRepository::new(pool.clone());
    let store_repo = StoreRepository::new(pool.clone());
    let service = ProductService::new(product_repo, store_repo);

    let result = service
        .create_product(CreateProductRequest {
            store_id: fake_store_id,
            sku: format!("SKU-{}", Uuid::new_v4()),
            name: "Product".to_string(),
            description: None,
            price: 10.0,
            stock_quantity: 5,
            category: None,
        })
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        markethub::error::AppError::NotFound(_)
    ));
}

#[tokio::test]
async fn product_list_by_store() {
    let pool = setup_test_db().await;
    let owner_id = create_test_user(&pool, &format!("prod3-{}@test.com", Uuid::new_v4())).await;
    let store = create_store(&pool, owner_id, &format!("store-{}", Uuid::new_v4()), false).await;

    // Create products
    create_product(&pool, store.id, &format!("SKU-{}", Uuid::new_v4()), 10.0, 5).await;
    create_product(
        &pool,
        store.id,
        &format!("SKU-{}", Uuid::new_v4()),
        20.0,
        10,
    )
    .await;

    let product_repo = ProductRepository::new(pool.clone());
    let store_repo = StoreRepository::new(pool.clone());
    let service = ProductService::new(product_repo, store_repo);

    let products = service.list_by_store(store.id, 10, 0).await.unwrap();
    assert!(products.len() >= 2);
}

#[tokio::test]
async fn product_update_success() {
    let pool = setup_test_db().await;
    let owner_id = create_test_user(&pool, &format!("prod4-{}@test.com", Uuid::new_v4())).await;
    let store = create_store(&pool, owner_id, &format!("store-{}", Uuid::new_v4()), false).await;
    let product = create_product(
        &pool,
        store.id,
        &format!("SKU-{}", Uuid::new_v4()),
        50.0,
        10,
    )
    .await;

    let product_repo = ProductRepository::new(pool.clone());
    let store_repo = StoreRepository::new(pool.clone());
    let service = ProductService::new(product_repo, store_repo);

    let result = service
        .update_product(
            product.id,
            UpdateProductRequest {
                name: Some("Updated Name".to_string()),
                description: Some("Updated description".to_string()),
                price: Some(75.0),
                stock_quantity: Some(20),
                category: None,
                is_active: Some(true),
            },
        )
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.stock_quantity, 20);
}

#[tokio::test]
async fn product_get_product() {
    let pool = setup_test_db().await;
    let owner_id = create_test_user(&pool, &format!("prod5-{}@test.com", Uuid::new_v4())).await;
    let store = create_store(&pool, owner_id, &format!("store-{}", Uuid::new_v4()), false).await;
    let product = create_product(
        &pool,
        store.id,
        &format!("SKU-{}", Uuid::new_v4()),
        30.0,
        15,
    )
    .await;

    let product_repo = ProductRepository::new(pool.clone());
    let store_repo = StoreRepository::new(pool.clone());
    let service = ProductService::new(product_repo, store_repo);

    let result = service.get_product(product.id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, product.id);
}

#[tokio::test]
async fn product_get_nonexistent() {
    let pool = setup_test_db().await;
    let fake_id = Uuid::new_v4();

    let product_repo = ProductRepository::new(pool.clone());
    let store_repo = StoreRepository::new(pool.clone());
    let service = ProductService::new(product_repo, store_repo);

    let result = service.get_product(fake_id).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        markethub::error::AppError::NotFound(_)
    ));
}

// ========== USER SERVICE TESTS ==========

#[tokio::test]
async fn user_get_profile_success() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool, &format!("profile-{}@test.com", Uuid::new_v4())).await;

    let user_repo = UserRepository::new(pool.clone());
    let service = UserService::new(user_repo);

    let result = service.get_profile(user_id).await;
    assert!(result.is_ok());
    let profile = result.unwrap();
    assert_eq!(profile.id, user_id);
}

#[tokio::test]
async fn user_get_profile_nonexistent() {
    let pool = setup_test_db().await;
    let fake_id = Uuid::new_v4();

    let user_repo = UserRepository::new(pool.clone());
    let service = UserService::new(user_repo);

    let result = service.get_profile(fake_id).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        markethub::error::AppError::NotFound(_)
    ));
}
