use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::routes::AppState;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn err(status: StatusCode, msg: &str) -> ApiError {
    (status, Json(serde_json::json!({"error": msg})))
}

fn internal(e: impl std::fmt::Display) -> ApiError {
    err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct UserAgentRow {
    user_agent: String,
    first_seen_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct BlockedUserAgentRow {
    user_agent: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct UserAgentBody {
    user_agent: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_group_user_agents)).route(
        "/blocked",
        get(list_group_blocked_user_agents)
            .post(add_group_blocked_user_agent)
            .delete(remove_group_blocked_user_agent),
    )
}

async fn list_group_user_agents(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<Vec<UserAgentRow>>, ApiError> {
    let rows = sqlx::query_as::<_, UserAgentRow>(
        "SELECT user_agent, first_seen_at FROM group_user_agents \
         WHERE group_id = $1 ORDER BY first_seen_at DESC",
    )
    .bind(group_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    Ok(Json(rows))
}

async fn list_group_blocked_user_agents(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<Vec<BlockedUserAgentRow>>, ApiError> {
    let rows = sqlx::query_as::<_, BlockedUserAgentRow>(
        "SELECT user_agent, created_at FROM group_blocked_user_agents \
         WHERE group_id = $1 ORDER BY created_at DESC",
    )
    .bind(group_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    Ok(Json(rows))
}

async fn add_group_blocked_user_agent(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Json(body): Json<UserAgentBody>,
) -> Result<StatusCode, ApiError> {
    sqlx::query(
        "INSERT INTO group_blocked_user_agents (group_id, user_agent) VALUES ($1, $2) \
         ON CONFLICT DO NOTHING",
    )
    .bind(group_id)
    .bind(&body.user_agent)
    .execute(&state.db)
    .await
    .map_err(internal)?;

    crate::cache::invalidate_group_all_keys(&state.redis, &state.db, group_id).await;

    Ok(StatusCode::CREATED)
}

async fn remove_group_blocked_user_agent(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Json(body): Json<UserAgentBody>,
) -> Result<StatusCode, ApiError> {
    sqlx::query("DELETE FROM group_blocked_user_agents WHERE group_id = $1 AND user_agent = $2")
        .bind(group_id)
        .bind(&body.user_agent)
        .execute(&state.db)
        .await
        .map_err(internal)?;

    crate::cache::invalidate_group_all_keys(&state.redis, &state.db, group_id).await;

    Ok(StatusCode::NO_CONTENT)
}
