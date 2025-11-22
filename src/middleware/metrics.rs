use std::time::Instant;

use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};

use crate::state::AppState;

pub async fn track_metrics(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(req).await;
    let status = response.status().as_u16();

    state
        .metrics
        .observe_http_request(method.as_str(), &path, status, start.elapsed());

    response
}
