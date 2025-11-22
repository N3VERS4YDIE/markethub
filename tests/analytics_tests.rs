use chrono::{Duration, TimeZone, Utc};
use markethub::{
    models::order::PaymentStatus,
    repositories::{AnalyticsRepository, StoreRepository},
    services::analytics_service::AnalyticsService,
};
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

struct AnalyticsFixture {
    store_id: Uuid,
    product_a: Uuid,
    product_b: Uuid,
    day_one: chrono::DateTime<Utc>,
    day_two: chrono::DateTime<Utc>,
    total_revenue: Decimal,
    average_order: Decimal,
}

impl AnalyticsFixture {
    async fn seed(pool: &PgPool) -> Self {
        let owner_id = Uuid::new_v4();
        let user_one = Uuid::new_v4();
        let user_two = Uuid::new_v4();
        let store_id = Uuid::new_v4();
        let product_a = Uuid::new_v4();
        let product_b = Uuid::new_v4();

        insert_user(pool, owner_id, "owner@markethub.dev").await;
        insert_user(pool, user_one, "buyer1@markethub.dev").await;
        insert_user(pool, user_two, "buyer2@markethub.dev").await;

        sqlx::query(
            r#"
            INSERT INTO stores (id, owner_id, name, slug, is_private)
            VALUES ($1, $2, $3, $4, false)
            "#,
        )
        .bind(store_id)
        .bind(owner_id)
        .bind("Gadget Hub")
        .bind("gadget-hub")
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            INSERT INTO products (id, store_id, sku, name, price, stock_quantity)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(product_a)
        .bind(store_id)
        .bind("SKU-A")
        .bind("Alpha Widget")
        .bind(Decimal::new(1500, 2))
        .bind(100)
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            INSERT INTO products (id, store_id, sku, name, price, stock_quantity)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(product_b)
        .bind(store_id)
        .bind("SKU-B")
        .bind("Beta Widget")
        .bind(Decimal::new(2000, 2))
        .bind(100)
        .execute(pool)
        .await
        .unwrap();

        let day_one = Utc.with_ymd_and_hms(2025, 1, 10, 12, 0, 0).unwrap();
        let day_two = Utc.with_ymd_and_hms(2025, 1, 11, 12, 0, 0).unwrap();

        create_order_with_item(
            pool,
            store_id,
            user_one,
            product_a,
            Decimal::new(1500, 2),
            2,
            "GRP-1001",
            "ORD-1001",
            day_one,
        )
        .await;

        create_order_with_item(
            pool,
            store_id,
            user_two,
            product_b,
            Decimal::new(2000, 2),
            1,
            "GRP-1002",
            "ORD-1002",
            day_two,
        )
        .await;

        let total_revenue = Decimal::new(3000, 2) + Decimal::new(2000, 2);
        let average_order = total_revenue / Decimal::from(2);

        Self {
            store_id,
            product_a,
            product_b,
            day_one,
            day_two,
            total_revenue,
            average_order,
        }
    }
}

async fn insert_user(pool: &PgPool, user_id: Uuid, email: &str) {
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, full_name)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(user_id)
    .bind(email)
    .bind("hashed")
    .bind("Test User")
    .execute(pool)
    .await
    .unwrap();
}

#[allow(clippy::too_many_arguments)]
async fn create_order_with_item(
    pool: &PgPool,
    store_id: Uuid,
    user_id: Uuid,
    product_id: Uuid,
    unit_price: Decimal,
    quantity: i32,
    group_number: &str,
    order_number: &str,
    created_at: chrono::DateTime<Utc>,
) {
    let order_group_id = Uuid::new_v4();
    let order_id = Uuid::new_v4();
    let order_item_id = Uuid::new_v4();
    let total_amount = unit_price * Decimal::from(quantity);

    sqlx::query(
        r#"
        INSERT INTO order_groups (id, user_id, group_number, total_amount, payment_status)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(order_group_id)
    .bind(user_id)
    .bind(group_number)
    .bind(total_amount)
    .bind(PaymentStatus::Paid)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO orders (
            id, order_group_id, user_id, store_id, order_number,
            subtotal, tax, discount, shipping_cost, total_amount, shipping_address
        ) VALUES (
            $1, $2, $3, $4, $5,
            $6, 0, 0, 0, $7, $8
        )
        "#,
    )
    .bind(order_id)
    .bind(order_group_id)
    .bind(user_id)
    .bind(store_id)
    .bind(order_number)
    .bind(total_amount)
    .bind(total_amount)
    .bind(json!({"line1": "123 Test St", "city": "Testville"}))
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        UPDATE orders SET created_at = $2 WHERE id = $1
        "#,
    )
    .bind(order_id)
    .bind(created_at)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO order_items (id, order_id, product_id, quantity, unit_price, subtotal)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(order_item_id)
    .bind(order_id)
    .bind(product_id)
    .bind(quantity)
    .bind(unit_price)
    .bind(total_amount)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "./migrations")]
async fn analytics_repository_reports_expected_metrics(pool: PgPool) -> sqlx::Result<()> {
    let fixture = AnalyticsFixture::seed(&pool).await;
    let repo = AnalyticsRepository::new(pool.clone());
    let since = fixture.day_one - Duration::days(1);

    let summary = repo
        .store_summary(fixture.store_id, since, 7)
        .await
        .expect("summary");

    assert_eq!(summary.total_orders, 2);
    assert_eq!(summary.unique_customers, 2);
    assert_eq!(summary.total_revenue, fixture.total_revenue);
    assert_eq!(summary.average_order_value, fixture.average_order);

    let trend = repo
        .store_sales_trend(fixture.store_id, since)
        .await
        .expect("trend");

    assert_eq!(trend.len(), 2);
    assert_eq!(trend[0].date, fixture.day_one.date_naive());
    assert_eq!(trend[0].order_count, 1);
    assert_eq!(trend[0].total_revenue, Decimal::new(3000, 2));
    assert_eq!(trend[1].date, fixture.day_two.date_naive());
    assert_eq!(trend[1].order_count, 1);
    assert_eq!(trend[1].total_revenue, Decimal::new(2000, 2));

    let top_products = repo
        .store_top_products(fixture.store_id, since, 5)
        .await
        .expect("top products");

    assert_eq!(top_products.len(), 2);
    assert_eq!(top_products[0].product_id, fixture.product_a);
    assert_eq!(top_products[0].units_sold, 2);
    assert_eq!(top_products[0].revenue, Decimal::new(3000, 2));
    assert_eq!(top_products[1].product_id, fixture.product_b);
    assert_eq!(top_products[1].units_sold, 1);
    assert_eq!(top_products[1].revenue, Decimal::new(2000, 2));

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn analytics_service_combines_all_sections(pool: PgPool) -> sqlx::Result<()> {
    let fixture = AnalyticsFixture::seed(&pool).await;
    let service = AnalyticsService::new(
        StoreRepository::new(pool.clone()),
        AnalyticsRepository::new(pool.clone()),
    );

    let response = service
        .store_analytics(fixture.store_id, 400, 5)
        .await
        .expect("service response");

    assert_eq!(response.summary.total_orders, 2);
    assert_eq!(response.summary.total_revenue, fixture.total_revenue);
    assert_eq!(response.sales_trend.len(), 2);
    assert_eq!(response.top_products[0].product_id, fixture.product_a);

    Ok(())
}
