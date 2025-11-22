use std::sync::Arc;

use crate::{metrics::Metrics, utils::jwt::JwtConfig};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt: Arc<JwtConfig>,
    pub metrics: Arc<Metrics>,
}

impl AppState {
    pub fn new(db: PgPool, jwt: JwtConfig, metrics: Arc<Metrics>) -> Self {
        Self {
            db,
            jwt: Arc::new(jwt),
            metrics,
        }
    }
}
