pub mod group_allowed_models;
pub mod group_key_allowed_models;
pub mod group_key_servers;
pub mod group_keys;
pub mod group_servers;
pub mod group_user_agents;
pub mod groups;
pub mod key_subscriptions;
pub mod logs;
pub mod models;
pub mod servers;
pub mod settings;
pub mod spam_detection;
pub mod subscription_plans;
pub mod token_usage;
pub mod ttft;
pub mod uptime;

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde::Deserialize;

use crate::routes::AppState;

#[derive(Debug, Deserialize)]
struct LoginRequest {
    token: String,
}

async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if input.token == state.admin_token {
        Ok(Json(serde_json::json!({"success": true})))
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid admin token"})),
        ))
    }
}

pub fn router(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .nest("/servers", servers::router())
        .nest("/groups", groups::router())
        .nest("/models", models::router())
        .nest("/subscription-plans", subscription_plans::router())
        .nest("/logs", logs::router())
        .nest("/ttft-stats", ttft::router())
        .nest("/token-usage", token_usage::router())
        .nest("/settings", settings::router())
        .nest("/spam-detection", spam_detection::router())
        .layer(axum::middleware::from_fn_with_state(
            state,
            crate::middleware::admin_auth::admin_auth,
        ));

    Router::new().route("/login", post(login)).merge(protected)
}
