use axum::Json;
use serde_json::{json, Value};

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "markethub",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn metrics() -> String {
    // Prometheus metrics will be implemented here
    "# HELP markethub_info MarketHub service info\n# TYPE markethub_info gauge\nmarkethub_info 1\n"
        .to_string()
}
