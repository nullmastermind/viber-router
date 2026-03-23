use axum::Router;
use sqlx::PgPool;

mod admin;
mod health;
mod proxy;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: deadpool_redis::Pool,
    pub admin_token: String,
    pub http_client: reqwest::Client,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", axum::routing::get(health::health_check))
        .nest("/api/admin", admin::router(state.clone()))
        .nest("/v1", proxy::router())
        .with_state(state)
}
