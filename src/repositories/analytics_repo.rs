use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::Result,
    models::store::{StoreAnalyticsSummary, StoreSalesPoint, StoreTopProduct},
};

#[derive(Clone)]
pub struct AnalyticsRepository {
    pool: PgPool,
}

impl AnalyticsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_summary(
        &self,
        store_id: Uuid,
        since: DateTime<Utc>,
        timeframe_days: i64,
    ) -> Result<StoreAnalyticsSummary> {
        let row = sqlx::query_as::<_, StoreSummaryRow>(
            r#"
            SELECT
                COUNT(*)::bigint AS total_orders,
                COALESCE(SUM(total_amount), 0) AS total_revenue,
                COALESCE(AVG(total_amount), 0) AS average_order_value,
                COUNT(DISTINCT user_id)::bigint AS unique_customers
            FROM orders
            WHERE store_id = $1 AND created_at >= $2
            "#,
        )
        .bind(store_id)
        .bind(since)
        .fetch_one(&self.pool)
        .await?;

        Ok(StoreAnalyticsSummary {
            total_orders: row.total_orders,
            total_revenue: row.total_revenue,
            average_order_value: row.average_order_value,
            unique_customers: row.unique_customers,
            timeframe_days,
        })
    }

    pub async fn store_sales_trend(
        &self,
        store_id: Uuid,
        since: DateTime<Utc>,
    ) -> Result<Vec<StoreSalesPoint>> {
        let rows = sqlx::query_as::<_, StoreSalesRow>(
            r#"
            SELECT
                DATE_TRUNC('day', created_at)::date AS bucket,
                COUNT(*)::bigint AS order_count,
                COALESCE(SUM(total_amount), 0) AS total_revenue
            FROM orders
            WHERE store_id = $1 AND created_at >= $2
            GROUP BY bucket
            ORDER BY bucket ASC
            "#,
        )
        .bind(store_id)
        .bind(since)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| StoreSalesPoint {
                date: row.bucket,
                order_count: row.order_count,
                total_revenue: row.total_revenue,
            })
            .collect())
    }

    pub async fn store_top_products(
        &self,
        store_id: Uuid,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<StoreTopProduct>> {
        let rows = sqlx::query_as::<_, StoreTopProductRow>(
            r#"
            SELECT
                oi.product_id,
                p.name AS product_name,
                SUM(oi.quantity)::bigint AS units_sold,
                COALESCE(SUM(oi.subtotal), 0) AS revenue
            FROM order_items oi
            INNER JOIN orders o ON oi.order_id = o.id
            INNER JOIN products p ON oi.product_id = p.id
            WHERE o.store_id = $1 AND o.created_at >= $2
            GROUP BY oi.product_id, p.name
            ORDER BY units_sold DESC
            LIMIT $3
            "#,
        )
        .bind(store_id)
        .bind(since)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| StoreTopProduct {
                product_id: row.product_id,
                product_name: row.product_name,
                units_sold: row.units_sold,
                revenue: row.revenue,
            })
            .collect())
    }
}

#[derive(sqlx::FromRow)]
struct StoreSummaryRow {
    total_orders: i64,
    total_revenue: Decimal,
    average_order_value: Decimal,
    unique_customers: i64,
}

#[derive(sqlx::FromRow)]
struct StoreSalesRow {
    bucket: NaiveDate,
    order_count: i64,
    total_revenue: Decimal,
}

#[derive(sqlx::FromRow)]
struct StoreTopProductRow {
    product_id: Uuid,
    product_name: String,
    units_sold: i64,
    revenue: Decimal,
}
