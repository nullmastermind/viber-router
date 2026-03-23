mod config;
mod db;
mod redis;
mod routes;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::Config::load()?;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&config.rust_log)),
        )
        .init();

    tracing::info!("Starting viber-router-api");

    let db_pool = db::create_pool(&config).await?;
    tracing::info!("Connected to PostgreSQL");

    let redis_pool = redis::create_pool(&config)?;
    tracing::info!("Redis pool created");

    let state = routes::AppState {
        db: db_pool,
        redis: redis_pool,
    };

    let app = routes::router(state);
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shut down");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install ctrl+c handler");
    tracing::info!("Shutdown signal received");
}
