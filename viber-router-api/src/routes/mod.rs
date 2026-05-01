use axum::Router;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

mod admin;
mod health;
pub mod key_parser;
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

    Router::new()
        .route("/health", axum::routing::get(health::health_check))
        .nest("/api/admin", admin::router(state.clone()))
        .nest("/api/public", public::router())
        .nest("/v1", proxy::router())
        .with_state(state)
        .fallback_service(serve_spa)
}
