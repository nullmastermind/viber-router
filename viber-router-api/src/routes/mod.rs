use axum::Router;
use sqlx::PgPool;
use tokio::sync::mpsc;

mod admin;
mod health;
pub mod key_parser;
mod proxy;

use crate::log_buffer::ProxyLogEntry;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: deadpool_redis::Pool,
    pub admin_token: String,
    pub http_client: reqwest::Client,
    pub log_tx: mpsc::Sender<ProxyLogEntry>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", axum::routing::get(health::health_check))
        .nest("/api/admin", admin::router(state.clone()))
        .nest("/v1", proxy::router())
        .with_state(state)
}
