mod cache;
mod circuit_breaker;
mod config;
mod db;
mod log_buffer;
mod middleware;
mod models;
mod partition;
mod redis;
mod routes;
mod sse_usage_parser;
mod subscription;
mod telegram_notifier;
mod ttft_buffer;
mod usage_buffer;

use std::time::Duration;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    partition::ensure_partitions(&db_pool, "proxy_logs").await;
    partition::ensure_partitions(&db_pool, "ttft_logs").await;
    partition::ensure_partitions(&db_pool, "token_usage_logs").await;
    tracing::info!("Partitions ensured");

    let redis_pool = redis::create_pool(&config)?;
    tracing::info!("Redis pool created");

    // Create log buffer
    let (log_tx, log_rx) = tokio::sync::mpsc::channel(10_000);

    // Create TTFT log buffer
    let (ttft_tx, ttft_rx) = tokio::sync::mpsc::channel(10_000);

    // Create token usage log buffer
    let (usage_tx, usage_rx) = tokio::sync::mpsc::channel(10_000);

    // Create and populate pricing cache
    let pricing_cache: routes::PricingCache = Arc::new(RwLock::new(HashMap::new()));
    routes::refresh_pricing_cache(&db_pool, &pricing_cache).await;
    tracing::info!("Pricing cache loaded");

    let state = routes::AppState {
        db: db_pool.clone(),
        redis: redis_pool,
        admin_token: config.admin_token,
        http_client: reqwest::Client::builder()
            .timeout(Duration::from_secs(8 * 3600))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(3600))
            .tcp_keepalive(Duration::from_secs(30))
            .tcp_nodelay(true)
            .http2_keep_alive_interval(Duration::from_secs(30))
            .http2_keep_alive_timeout(Duration::from_secs(10))
            .http2_keep_alive_while_idle(true)
            .build()
            .expect("Failed to build HTTP client"),
        log_tx,
        ttft_tx,
        usage_tx,
        pricing_cache: pricing_cache.clone(),
    };

    // Spawn log buffer flush task
    let flush_handle = tokio::spawn(log_buffer::flush_task(log_rx, db_pool.clone()));

    // Spawn TTFT buffer flush task
    let ttft_flush_handle = tokio::spawn(ttft_buffer::flush_task(ttft_rx, db_pool.clone()));

    // Spawn token usage buffer flush task
    let usage_flush_handle = tokio::spawn(usage_buffer::flush_task(usage_rx, db_pool.clone()));

    // Spawn daily partition maintenance
    let retention_days = config.log_retention_days;
    let partition_pool = db_pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(86400));
        interval.tick().await; // skip immediate tick
        loop {
            interval.tick().await;
            partition::ensure_partitions(&partition_pool, "proxy_logs").await;
            partition::ensure_partitions(&partition_pool, "ttft_logs").await;
            partition::ensure_partitions(&partition_pool, "token_usage_logs").await;
            partition::drop_expired_partitions(&partition_pool, "proxy_logs", retention_days).await;
            partition::drop_expired_partitions(&partition_pool, "ttft_logs", retention_days).await;
            partition::drop_expired_partitions(&partition_pool, "token_usage_logs", retention_days).await;
            tracing::info!("Daily partition maintenance complete");
        }
    });

    // Spawn pricing cache refresh task (every 60 seconds)
    let pricing_pool = db_pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        interval.tick().await; // skip immediate tick
        loop {
            interval.tick().await;
            routes::refresh_pricing_cache(&pricing_pool, &pricing_cache).await;
        }
    });

    let app = routes::router(state.clone());
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Drop state to close the log channel sender, then drain the flush tasks
    drop(state);
    let _ = flush_handle.await;
    let _ = ttft_flush_handle.await;
    let _ = usage_flush_handle.await;

    tracing::info!("Server shut down");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install ctrl+c handler");
    tracing::info!("Shutdown signal received");
}
