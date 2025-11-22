use crate::config::Config;
use crate::handlers;
use crate::metrics::Metrics;
use crate::middleware::metrics::track_metrics;
use crate::state::AppState;
use crate::utils::jwt::JwtConfig;
use axum::middleware;
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, sync::Arc};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};

pub async fn run(config: Config) -> anyhow::Result<()> {
    // Database connection pool
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    tracing::info!("Database connected and migrations applied");

    let jwt_config = JwtConfig::new(&config.jwt_secret, config.jwt_expiration_hours);
    let metrics = Arc::new(Metrics::default());
    let state = AppState::new(db_pool.clone(), jwt_config, metrics.clone());

    // Build router
    let app = handlers::api_router()
        .layer(middleware::from_fn_with_state(state.clone(), track_metrics))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true)),
        )
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
