use crate::config::Config;
use crate::handlers;
use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
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

    // Build router
    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/metrics", get(handlers::metrics))
        // API routes will be added here
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true)),
        )
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(db_pool);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
