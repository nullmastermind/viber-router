use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{post, put},
};
use uuid::Uuid;

use crate::models::{AssignServer, GroupServer, ReorderServers, UpdateAssignment};
use crate::routes::AppState;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn err(status: StatusCode, msg: &str) -> ApiError {
    (status, Json(serde_json::json!({"error": msg})))
}

fn internal(e: impl std::fmt::Display) -> ApiError {
    err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
}

/// Validate circuit breaker fields: all-or-nothing, each >= 1.
fn validate_cb_fields(
    max_failures: Option<Option<i32>>,
    window_seconds: Option<Option<i32>>,
    cooldown_seconds: Option<Option<i32>>,
) -> Result<(), ApiError> {
    let provided: Vec<&Option<i32>> = [&max_failures, &window_seconds, &cooldown_seconds]
        .into_iter()
        .filter_map(|f| f.as_ref())
        .collect();

    if provided.is_empty() {
        return Ok(());
    }

    if provided.len() != 3 {
        return Err(err(StatusCode::BAD_REQUEST, "Circuit breaker fields must be all set or all null"));
    }

    let all_null = provided.iter().all(|v| v.is_none());
    let all_some = provided.iter().all(|v| v.is_some());

    if !all_null && !all_some {
        return Err(err(StatusCode::BAD_REQUEST, "Circuit breaker fields must be all set or all null"));
    }

    if all_some {
        for v in &provided {
            if let Some(val) = v
                && *val < 1
            {
                return Err(err(StatusCode::BAD_REQUEST, "Circuit breaker values must be >= 1"));
            }
        }
    }

    Ok(())
}

/// Validate rate limit fields: all-or-nothing, each >= 1.
fn validate_rate_limit_fields(
    max_requests: Option<Option<i32>>,
    rate_window_seconds: Option<Option<i32>>,
) -> Result<(), ApiError> {
    let provided: Vec<&Option<i32>> = [&max_requests, &rate_window_seconds]
        .into_iter()
        .filter_map(|f| f.as_ref())
        .collect();

    if provided.is_empty() {
        return Ok(());
    }

    if provided.len() != 2 {
        return Err(err(StatusCode::BAD_REQUEST, "Rate limit fields must be all set or all null"));
    }

    let all_null = provided.iter().all(|v| v.is_none());
    let all_some = provided.iter().all(|v| v.is_some());

    if !all_null && !all_some {
        return Err(err(StatusCode::BAD_REQUEST, "Rate limit fields must be all set or all null"));
    }

    if all_some {
        for v in &provided {
            if let Some(val) = v
                && *val < 1
            {
                return Err(err(StatusCode::BAD_REQUEST, "Rate limit values must be >= 1"));
            }
        }
    }

    Ok(())
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(assign_server))
        .route("/reorder", put(reorder_priorities))
        .route("/{server_id}", put(update_assignment).delete(remove_server))
}

async fn assign_server(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Json(input): Json<AssignServer>,
) -> Result<(StatusCode, Json<GroupServer>), ApiError> {
    let mappings = input.model_mappings.unwrap_or(serde_json::json!({}));

    let gs = sqlx::query_as::<_, GroupServer>(
        "INSERT INTO group_servers (group_id, server_id, priority, model_mappings, is_enabled, cb_max_failures, cb_window_seconds, cb_cooldown_seconds) \
         VALUES ($1, $2, $3, $4, true, NULL, NULL, NULL) RETURNING *",
    )
    .bind(group_id)
    .bind(input.server_id)
    .bind(input.priority)
    .bind(&mappings)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") || e.to_string().contains("unique") {
            err(StatusCode::CONFLICT, "Server already assigned to this group")
        } else {
            internal(e)
        }
    })?;

    invalidate_group_cache(&state, group_id).await;
    Ok((StatusCode::CREATED, Json(gs)))
}

async fn update_assignment(
    State(state): State<AppState>,
    Path((group_id, server_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateAssignment>,
) -> Result<Json<GroupServer>, ApiError> {
    // Validate circuit breaker fields
    validate_cb_fields(input.cb_max_failures, input.cb_window_seconds, input.cb_cooldown_seconds)?;

    // Validate rate limit fields
    validate_rate_limit_fields(input.max_requests, input.rate_window_seconds)?;

    // Validate non-negative rate fields
    for (field, val) in [
        ("rate_input", &input.rate_input),
        ("rate_output", &input.rate_output),
        ("rate_cache_write", &input.rate_cache_write),
        ("rate_cache_read", &input.rate_cache_read),
    ] {
        if let Some(Some(v)) = val
            && *v < 0.0
        {
            return Err(err(StatusCode::BAD_REQUEST, &format!("{field} must be non-negative")));
        }
    }

    // Determine whether to update CB fields
    let (update_cb_max, cb_max_val) = match input.cb_max_failures {
        Some(v) => (true, v),
        None => (false, None),
    };
    let (update_cb_window, cb_window_val) = match input.cb_window_seconds {
        Some(v) => (true, v),
        None => (false, None),
    };
    let (update_cb_cooldown, cb_cooldown_val) = match input.cb_cooldown_seconds {
        Some(v) => (true, v),
        None => (false, None),
    };

    // Determine whether to update rate fields
    let (update_rate_input, rate_input_val) = match input.rate_input {
        Some(v) => (true, v),
        None => (false, None),
    };
    let (update_rate_output, rate_output_val) = match input.rate_output {
        Some(v) => (true, v),
        None => (false, None),
    };
    let (update_rate_cw, rate_cw_val) = match input.rate_cache_write {
        Some(v) => (true, v),
        None => (false, None),
    };
    let (update_rate_cr, rate_cr_val) = match input.rate_cache_read {
        Some(v) => (true, v),
        None => (false, None),
    };

    // Determine whether to update rate limit fields
    let (update_max_requests, max_requests_val) = match input.max_requests {
        Some(v) => (true, v),
        None => (false, None),
    };
    let (update_rate_window, rate_window_val) = match input.rate_window_seconds {
        Some(v) => (true, v),
        None => (false, None),
    };

    let gs = sqlx::query_as::<_, GroupServer>(
        "UPDATE group_servers SET \
         priority = COALESCE($1, priority), \
         model_mappings = COALESCE($2, model_mappings), \
         is_enabled = COALESCE($3, is_enabled), \
         cb_max_failures = CASE WHEN $6 THEN $7 ELSE cb_max_failures END, \
         cb_window_seconds = CASE WHEN $8 THEN $9 ELSE cb_window_seconds END, \
         cb_cooldown_seconds = CASE WHEN $10 THEN $11 ELSE cb_cooldown_seconds END, \
         rate_input = CASE WHEN $12 THEN $13 ELSE rate_input END, \
         rate_output = CASE WHEN $14 THEN $15 ELSE rate_output END, \
         rate_cache_write = CASE WHEN $16 THEN $17 ELSE rate_cache_write END, \
         rate_cache_read = CASE WHEN $18 THEN $19 ELSE rate_cache_read END, \
         max_requests = CASE WHEN $20 THEN $21 ELSE max_requests END, \
         rate_window_seconds = CASE WHEN $22 THEN $23 ELSE rate_window_seconds END \
         WHERE group_id = $4 AND server_id = $5 RETURNING *",
    )
    .bind(input.priority)
    .bind(&input.model_mappings)
    .bind(input.is_enabled)
    .bind(group_id)
    .bind(server_id)
    .bind(update_cb_max)
    .bind(cb_max_val)
    .bind(update_cb_window)
    .bind(cb_window_val)
    .bind(update_cb_cooldown)
    .bind(cb_cooldown_val)
    .bind(update_rate_input)
    .bind(rate_input_val)
    .bind(update_rate_output)
    .bind(rate_output_val)
    .bind(update_rate_cw)
    .bind(rate_cw_val)
    .bind(update_rate_cr)
    .bind(rate_cr_val)
    .bind(update_max_requests)
    .bind(max_requests_val)
    .bind(update_rate_window)
    .bind(rate_window_val)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Assignment not found"))?;

    invalidate_group_cache(&state, group_id).await;
    Ok(Json(gs))
}

async fn remove_server(
    State(state): State<AppState>,
    Path((group_id, server_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query(
        "DELETE FROM group_servers WHERE group_id = $1 AND server_id = $2",
    )
    .bind(group_id)
    .bind(server_id)
    .execute(&state.db)
    .await
    .map_err(internal)?;

    if result.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Assignment not found"));
    }

    invalidate_group_cache(&state, group_id).await;
    Ok(StatusCode::NO_CONTENT)
}

async fn reorder_priorities(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Json(input): Json<ReorderServers>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await.map_err(internal)?;

    for (i, server_id) in input.server_ids.iter().enumerate() {
        sqlx::query(
            "UPDATE group_servers SET priority = $1 WHERE group_id = $2 AND server_id = $3",
        )
        .bind((i + 1) as i32)
        .bind(group_id)
        .bind(server_id)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    }

    tx.commit().await.map_err(internal)?;

    invalidate_group_cache(&state, group_id).await;
    Ok(StatusCode::OK)
}

async fn invalidate_group_cache(state: &AppState, group_id: Uuid) {
    crate::cache::invalidate_group_all_keys(&state.redis, &state.db, group_id).await;
}
