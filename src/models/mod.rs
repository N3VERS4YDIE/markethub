use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod order;
pub mod permission;
pub mod product;
pub mod store;
pub mod user;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            meta: ResponseMeta {
                timestamp: Utc::now(),
            },
        }
    }
}
