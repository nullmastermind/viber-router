pub mod servers;
pub mod groups;
pub mod group_servers;
pub mod logs;
pub mod ttft;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::post,
};
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
        .nest("/logs", logs::router())
        .nest("/ttft-stats", ttft::router())
        .layer(axum::middleware::from_fn_with_state(
            state,
            crate::middleware::admin_auth::admin_auth,
        ));

    Router::new()
        .route("/login", post(login))
        .merge(protected)
}
