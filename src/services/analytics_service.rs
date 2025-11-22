use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::store::StoreAnalyticsResponse,
    repositories::{AnalyticsRepository, StoreRepository},
};

#[derive(Clone)]
pub struct AnalyticsService {
    stores: StoreRepository,
    analytics: AnalyticsRepository,
}

impl AnalyticsService {
    pub fn new(stores: StoreRepository, analytics: AnalyticsRepository) -> Self {
        Self { stores, analytics }
    }

    pub async fn store_analytics(
        &self,
        store_id: Uuid,
        timeframe_days: i64,
        top_products_limit: i64,
    ) -> crate::Result<StoreAnalyticsResponse> {
        let store = self
            .stores
            .find_by_id(store_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Store not found".into()))?;

        let since = Utc::now() - Duration::days(timeframe_days);

        let summary = self
            .analytics
            .store_summary(store.id, since, timeframe_days)
            .await?;

        let sales_trend = self.analytics.store_sales_trend(store.id, since).await?;

        let top_products = self
            .analytics
            .store_top_products(store.id, since, top_products_limit)
            .await?;

        Ok(StoreAnalyticsResponse {
            summary,
            sales_trend,
            top_products,
        })
    }
}
