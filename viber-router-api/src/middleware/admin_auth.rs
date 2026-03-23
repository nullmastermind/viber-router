use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::routes::AppState;

pub async fn admin_auth(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Authorization header required"})),
            )
                .into_response();
        }
        Some(header) => {
            let token = header.strip_prefix("Bearer ").unwrap_or(header);
            if token != state.admin_token {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error": "Invalid admin token"})),
                )
                    .into_response();
            }
        }
    }

    next.run(req).await
}
