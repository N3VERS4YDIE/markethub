use std::{sync::Arc, time::Duration};

use once_cell::sync::Lazy;
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry, TextEncoder,
};

static HTTP_DURATION_BUCKETS: Lazy<Vec<f64>> =
    Lazy::new(|| vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]);

#[derive(Clone)]
pub struct Metrics {
    registry: Registry,
    http_requests_total: IntCounterVec,
    http_request_duration_seconds: HistogramVec,
}

impl Metrics {
    pub fn new() -> Self {
        let registry = Registry::new_custom(Some("markethub".into()), None)
            .expect("metrics registry should initialize");

        let http_requests_total = IntCounterVec::new(
            Opts::new(
                "http_requests_total",
                "Total count of HTTP requests handled by method/path/status",
            ),
            &["method", "path", "status"],
        )
        .expect("counter vec should initialize");

        let http_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request latencies in seconds",
            )
            .buckets(HTTP_DURATION_BUCKETS.clone()),
            &["method", "path"],
        )
        .expect("histogram vec should initialize");

        registry
            .register(Box::new(http_requests_total.clone()))
            .expect("registry should register counter");
        registry
            .register(Box::new(http_request_duration_seconds.clone()))
            .expect("registry should register histogram");

        Self {
            registry,
            http_requests_total,
            http_request_duration_seconds,
        }
    }

    pub fn observe_http_request(&self, method: &str, path: &str, status: u16, duration: Duration) {
        let status_label = status.to_string();
        self.http_requests_total
            .with_label_values(&[method, path, &status_label])
            .inc();
        self.http_request_duration_seconds
            .with_label_values(&[method, path])
            .observe(duration.as_secs_f64());
    }

    pub fn encode(&self) -> Result<String, prometheus::Error> {
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        TextEncoder::new().encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer).into_owned())
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedMetrics = Arc<Metrics>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_encode_contains_observed_values() {
        let metrics = Metrics::new();
        metrics.observe_http_request("GET", "/health", 200, Duration::from_millis(25));
        metrics.observe_http_request("GET", "/health", 200, Duration::from_millis(50));

        let encoded = metrics.encode().expect("metrics should encode");
        assert!(encoded.contains("http_requests_total"));
        assert!(encoded.contains("/health"));
        assert!(encoded.contains("GET"));
    }
}
