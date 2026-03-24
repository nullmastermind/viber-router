mod cache;
mod config;
mod db;
mod log_buffer;
mod middleware;
mod models;
mod partition;
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

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await?;
    tracing::info!("Migrations applied");

    // Ensure partitions exist for current and next month
    partition::ensure_partitions(&db_pool).await;
    tracing::info!("Partitions ensured");

    let redis_pool = redis::create_pool(&config)?;
    tracing::info!("Redis pool created");

    // Create log buffer
    let (log_tx, log_rx) = tokio::sync::mpsc::channel(10_000);

    let state = routes::AppState {
        db: db_pool.clone(),
        redis: redis_pool,
        admin_token: config.admin_token,
        http_client: reqwest::Client::new(),
        log_tx,
    };

    // Spawn log buffer flush task
    let flush_handle = tokio::spawn(log_buffer::flush_task(log_rx, db_pool.clone()));

    // Spawn daily partition maintenance
    let retention_days = config.log_retention_days;
    let partition_pool = db_pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(86400));
        interval.tick().await; // skip immediate tick
        loop {
            interval.tick().await;
            partition::ensure_partitions(&partition_pool).await;
            partition::drop_expired_partitions(&partition_pool, retention_days).await;
            tracing::info!("Daily partition maintenance complete");
        }
    });

    let app = routes::router(state.clone());
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Drop state to close the log channel sender, then drain the flush task
    drop(state);
    let _ = flush_handle.await;

    tracing::info!("Server shut down");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install ctrl+c handler");
    tracing::info!("Shutdown signal received");
}
