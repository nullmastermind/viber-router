use axum::Router;
use axum::http::{HeaderName, Method};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

pub mod admin;
mod health;
pub mod key_parser;
mod llm;
mod proxy;
mod public;

use crate::log_buffer::ProxyLogEntry;
use crate::ttft_buffer::TtftLogEntry;
use crate::uptime_buffer::UptimeCheckEntry;
use crate::usage_buffer::TokenUsageEntry;

#[derive(Debug, Clone)]
pub struct ModelPricing {
    pub input_1m_usd: f64,
    pub output_1m_usd: f64,
    pub cache_write_1m_usd: f64,
    pub cache_read_1m_usd: f64,
}

pub type PricingCache = Arc<RwLock<HashMap<String, ModelPricing>>>;
pub type UnlockedServers = Arc<RwLock<HashSet<Uuid>>>;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: deadpool_redis::Pool,
    pub admin_token: String,
    pub http_client: reqwest::Client,
    pub log_tx: mpsc::Sender<ProxyLogEntry>,
    pub ttft_tx: mpsc::Sender<TtftLogEntry>,
    pub usage_tx: mpsc::Sender<TokenUsageEntry>,
    pub uptime_tx: mpsc::Sender<UptimeCheckEntry>,
    pub pricing_cache: PricingCache,
    pub unlocked_servers: UnlockedServers,
}

pub async fn refresh_pricing_cache(db: &PgPool, cache: &PricingCache) {
    let rows = sqlx::query_as::<_, (String, Option<f64>, Option<f64>, Option<f64>, Option<f64>)>(
        "SELECT name, input_1m_usd, output_1m_usd, cache_write_1m_usd, cache_read_1m_usd FROM models",
    )
    .fetch_all(db)
    .await;

    match rows {
        Ok(models) => {
            let mut map = HashMap::new();
            for (name, input, output, cache_write, cache_read) in models {
                if let (Some(i), Some(o)) = (input, output) {
                    map.insert(
                        name,
                        ModelPricing {
                            input_1m_usd: i,
                            output_1m_usd: o,
                            cache_write_1m_usd: cache_write.unwrap_or(0.0),
                            cache_read_1m_usd: cache_read.unwrap_or(0.0),
                        },
                    );
                }
            }
            *cache.write().await = map;
            tracing::debug!("Refreshed pricing cache");
        }
        Err(e) => tracing::warn!("Failed to refresh pricing cache: {e}"),
    }
}

pub fn router(state: AppState) -> Router {
    let spa_dir = std::env::var("SPA_DIR").unwrap_or_else(|_| "./dist/spa".into());
    let index = format!("{}/index.html", spa_dir);
    let serve_spa = ServeDir::new(&spa_dir).not_found_service(ServeFile::new(&index));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-api-key"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("anthropic-version"),
            HeaderName::from_static("anthropic-beta"),
            HeaderName::from_static("anthropic-dangerous-direct-browser-access"),
            HeaderName::from_static("x-stainless-lang"),
            HeaderName::from_static("x-stainless-package-version"),
            HeaderName::from_static("x-stainless-os"),
            HeaderName::from_static("x-stainless-arch"),
            HeaderName::from_static("x-stainless-runtime"),
            HeaderName::from_static("x-stainless-runtime-version"),
            HeaderName::from_static("x-stainless-retry-count"),
            HeaderName::from_static("x-stainless-timeout"),
            HeaderName::from_static("x-stainless-helper-method"),
        ])
        .expose_headers([
            HeaderName::from_static("request-id"),
            HeaderName::from_static("anthropic-ratelimit-requests-limit"),
            HeaderName::from_static("anthropic-ratelimit-requests-remaining"),
            HeaderName::from_static("anthropic-ratelimit-requests-reset"),
            HeaderName::from_static("anthropic-ratelimit-tokens-limit"),
            HeaderName::from_static("anthropic-ratelimit-tokens-remaining"),
            HeaderName::from_static("anthropic-ratelimit-tokens-reset"),
        ])
        .max_age(Duration::from_secs(86400));

    Router::new()
        .route("/health", axum::routing::get(health::health_check))
        .nest("/api/admin", admin::router(state.clone()))
        .nest("/api/public", public::router())
        .nest("/api/v1/llm", llm::router())
        .nest("/v1", proxy::router().layer(cors))
        .with_state(state)
        .fallback_service(serve_spa)
}
