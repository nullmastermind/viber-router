use axum::Router;
use sqlx::PgPool;
use tokio::sync::mpsc;
use tower_http::services::{ServeDir, ServeFile};

mod admin;
mod health;
pub mod key_parser;
mod proxy;

use crate::log_buffer::ProxyLogEntry;
use crate::ttft_buffer::TtftLogEntry;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: deadpool_redis::Pool,
    pub admin_token: String,
    pub http_client: reqwest::Client,
    pub log_tx: mpsc::Sender<ProxyLogEntry>,
    pub ttft_tx: mpsc::Sender<TtftLogEntry>,
}

pub fn router(state: AppState) -> Router {
    let spa_dir = std::env::var("SPA_DIR").unwrap_or_else(|_| "./dist/spa".into());
    let index = format!("{}/index.html", spa_dir);
    let serve_spa = ServeDir::new(&spa_dir).not_found_service(ServeFile::new(&index));

    Router::new()
        .route("/health", axum::routing::get(health::health_check))
        .nest("/api/admin", admin::router(state.clone()))
        .nest("/v1", proxy::router())
        .with_state(state)
        .fallback_service(serve_spa)
}
