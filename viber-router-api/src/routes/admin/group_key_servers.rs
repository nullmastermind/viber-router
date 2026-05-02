use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
};
use uuid::Uuid;

use crate::models::{AssignKeyServer, GroupKey, GroupServerDetail};
use crate::routes::AppState;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn err(status: StatusCode, msg: &str) -> ApiError {
    (status, Json(serde_json::json!({"error": msg})))
}

fn internal(e: impl std::fmt::Display) -> ApiError {
    err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_key_servers).post(assign_key_servers))
        .route("/{server_id}", delete(remove_key_server))
}

async fn list_key_servers(
    State(state): State<AppState>,
    Path((_group_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<GroupServerDetail>>, ApiError> {
    let servers = sqlx::query_as::<_, GroupServerDetail>(
        "SELECT gs.server_id, s.short_id, s.name as server_name, s.base_url, s.api_key, s.system_prompt, s.remove_thinking, gs.priority, gs.model_mappings, gs.is_enabled, \
         gs.cb_max_failures, gs.cb_window_seconds, gs.cb_cooldown_seconds, \
         gs.rate_input, gs.rate_output, gs.rate_cache_write, gs.rate_cache_read, \
         gs.max_requests, gs.rate_window_seconds, gs.normalize_cache_read, gs.max_input_tokens, gs.min_input_tokens, gs.supported_models \
         FROM group_key_servers gks \
         JOIN group_servers gs ON gs.server_id = gks.server_id AND gs.group_id = (SELECT group_id FROM group_keys WHERE id = $1) \
         JOIN servers s ON s.id = gks.server_id \
         WHERE gks.group_key_id = $1 \
         ORDER BY gs.priority",
    )
    .bind(key_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    Ok(Json(servers))
}

async fn assign_key_servers(
    State(state): State<AppState>,
    Path((group_id, key_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<AssignKeyServer>,
) -> Result<(StatusCode, Json<Vec<GroupServerDetail>>), ApiError> {
    if input.server_ids.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "server_ids must not be empty"));
    }

    // Look up the sub-key's api_key for cache invalidation
    let sub_key: Option<GroupKey> =
        sqlx::query_as::<_, GroupKey>("SELECT * FROM group_keys WHERE id = $1 AND group_id = $2")
            .bind(key_id)
            .bind(group_id)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?;

    let sub_key = sub_key.ok_or_else(|| err(StatusCode::NOT_FOUND, "Key not found"))?;

    // Validate that the key is a sub-key (not a master key) -- master key has no group_key_id
    // (the route is under group_keys so this is always a sub-key)

    // Validate each server_id is in the group's server list
    for server_id in &input.server_ids {
        let in_group: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM group_servers WHERE group_id = $1 AND server_id = $2)",
        )
        .bind(group_id)
        .bind(server_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

        if !in_group {
            return Err(err(
                StatusCode::BAD_REQUEST,
                &format!("Server {} is not in the group's server chain", server_id),
            ));
        }
    }

    // Insert all assignments, collecting results
    let mut assigned_servers: Vec<GroupServerDetail> = Vec::new();
    for server_id in &input.server_ids {
        let result =
            sqlx::query("INSERT INTO group_key_servers (group_key_id, server_id) VALUES ($1, $2)")
                .bind(key_id)
                .bind(server_id)
                .execute(&state.db)
                .await;

        match result {
            Ok(_) => {}
            Err(e)
                if e.to_string().contains("duplicate key") || e.to_string().contains("unique") =>
            {
                // Silently skip duplicates
            }
            Err(e) => return Err(internal(e)),
        }

        // Fetch the server detail for response
        let detail: Option<GroupServerDetail> = sqlx::query_as::<_, GroupServerDetail>(
            "SELECT gs.server_id, s.short_id, s.name as server_name, s.base_url, s.api_key, s.system_prompt, s.remove_thinking, gs.priority, gs.model_mappings, gs.is_enabled, \
             gs.cb_max_failures, gs.cb_window_seconds, gs.cb_cooldown_seconds, \
             gs.rate_input, gs.rate_output, gs.rate_cache_write, gs.rate_cache_read, \
             gs.max_requests, gs.rate_window_seconds, gs.normalize_cache_read, gs.max_input_tokens, gs.min_input_tokens, gs.supported_models \
             FROM group_servers gs JOIN servers s ON s.id = gs.server_id \
             WHERE gs.group_id = $1 AND gs.server_id = $2",
        )
        .bind(group_id)
        .bind(server_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?;

        if let Some(d) = detail {
            assigned_servers.push(d);
        }
    }

    // Invalidate only the sub-key's config cache
    crate::cache::invalidate_group_config(&state.redis, &sub_key.api_key).await;

    Ok((StatusCode::CREATED, Json(assigned_servers)))
}

async fn remove_key_server(
    State(state): State<AppState>,
    Path((group_id, key_id, server_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    // Look up the sub-key's api_key for cache invalidation
    let sub_key: Option<GroupKey> =
        sqlx::query_as::<_, GroupKey>("SELECT * FROM group_keys WHERE id = $1 AND group_id = $2")
            .bind(key_id)
            .bind(group_id)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?;

    let sub_key = sub_key.ok_or_else(|| err(StatusCode::NOT_FOUND, "Key not found"))?;

    let result =
        sqlx::query("DELETE FROM group_key_servers WHERE group_key_id = $1 AND server_id = $2")
            .bind(key_id)
            .bind(server_id)
            .execute(&state.db)
            .await
            .map_err(internal)?;

    if result.rows_affected() == 0 {
        return Err(err(
            StatusCode::NOT_FOUND,
            "Server not assigned to this key",
        ));
    }

    // Invalidate only the sub-key's config cache
    crate::cache::invalidate_group_config(&state.redis, &sub_key.api_key).await;

    Ok(StatusCode::NO_CONTENT)
}
