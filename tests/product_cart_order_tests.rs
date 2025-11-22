mod common;

use markethub::{
    error::AppError,
    models::order::{AddCartItemRequest, CheckoutRequest},
    repositories::{CartRepository, OrderRepository, ProductRepository},
    services::{cart_service::CartService, order_service::OrderService},
};
use rust_decimal::Decimal;
use sqlx::{query, PgPool};

fn cart_service(pool: &PgPool) -> CartService {
    CartService::new(
        CartRepository::new(pool.clone()),
        ProductRepository::new(pool.clone()),
    )
}

fn order_service(pool: &PgPool) -> OrderService {
    OrderService::new(
        OrderRepository::new(pool.clone()),
        ProductRepository::new(pool.clone()),
        CartRepository::new(pool.clone()),
    )
}

#[sqlx::test(migrations = "./migrations")]
async fn cart_service_validates_product_constraints(pool: PgPool) {
    let owner = common::insert_user(&pool, "cart-owner@markethub.dev").await;
    let shopper = common::insert_user(&pool, "shopper@markethub.dev").await;
    let store = common::create_store(&pool, owner.id, "cart-store", false).await;

    let product = common::create_product(&pool, store.id, "SKU-CART", 49.99, 10).await;
    let carts = cart_service(&pool);

    carts
        .add_item(
            shopper.id,
            AddCartItemRequest {
                product_id: product.id,
                quantity: 1,
            },
        )
        .await
        .expect("first insert should succeed");

    query("UPDATE products SET is_active = false WHERE id = $1")
        .bind(product.id)
        .execute(&pool)
        .await
        .unwrap();

    let err = carts
        .add_item(
            shopper.id,
            AddCartItemRequest {
                product_id: product.id,
                quantity: 1,
            },
        )
        .await
        .expect_err("inactive products should not be addable");
    assert!(matches!(err, AppError::BadRequest(_)));

    query("UPDATE products SET is_active = true, stock_quantity = 1 WHERE id = $1")
        .bind(product.id)
        .execute(&pool)
        .await
        .unwrap();

    let err = carts
        .add_item(
            shopper.id,
            AddCartItemRequest {
                product_id: product.id,
                quantity: 5,
            },
        )
        .await
        .expect_err("insufficient stock should be rejected");
    assert!(matches!(err, AppError::Conflict(_)));
}

#[sqlx::test(migrations = "./migrations")]
async fn checkout_groups_orders_and_clears_cart(pool: PgPool) {
    let owner = common::insert_user(&pool, "checkout-owner@markethub.dev").await;
    let shopper = common::insert_user(&pool, "checkout-shopper@markethub.dev").await;
    let store_a = common::create_store(&pool, owner.id, "checkout-store-a", false).await;
    let store_b = common::create_store(&pool, owner.id, "checkout-store-b", false).await;

    let product_a = common::create_product(&pool, store_a.id, "SKU-A", 25.0, 10).await;
    let product_b = common::create_product(&pool, store_b.id, "SKU-B", 15.0, 8).await;

    let carts = cart_service(&pool);
    let orders = order_service(&pool);

    carts
        .add_item(
            shopper.id,
            AddCartItemRequest {
                product_id: product_a.id,
                quantity: 2,
            },
        )
        .await
        .unwrap();
    carts
        .add_item(
            shopper.id,
            AddCartItemRequest {
                product_id: product_b.id,
                quantity: 1,
            },
        )
        .await
        .unwrap();

    let summary = orders
        .checkout(
            shopper.id,
            CheckoutRequest {
                shipping_address: common::shipping_address(),
            },
        )
        .await
        .unwrap();

    assert_eq!(summary.orders.len(), 2, "one order per store");
    let store_ids: Vec<_> = summary.orders.iter().map(|o| o.store_id).collect();
    assert!(store_ids.contains(&store_a.id));
    assert!(store_ids.contains(&store_b.id));

    let total_orders: Decimal = summary.orders.iter().map(|order| order.total_amount).sum();
    assert_eq!(summary.order_group.total_amount, total_orders);
    assert_eq!(summary.order_group.user_id, shopper.id);

    let cart_items = CartRepository::new(pool.clone())
        .list_with_products(shopper.id)
        .await
        .unwrap();
    assert!(
        cart_items.is_empty(),
        "cart should be cleared after checkout"
    );

    let products = ProductRepository::new(pool.clone());
    let updated_a = products.find_by_id(product_a.id).await.unwrap().unwrap();
    let updated_b = products.find_by_id(product_b.id).await.unwrap().unwrap();
    assert_eq!(updated_a.stock_quantity, 8);
    assert_eq!(updated_b.stock_quantity, 7);
}
