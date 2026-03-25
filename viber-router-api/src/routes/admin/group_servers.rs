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
        "INSERT INTO group_servers (group_id, server_id, priority, model_mappings, is_enabled) \
         VALUES ($1, $2, $3, $4, true) RETURNING *",
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
    let gs = sqlx::query_as::<_, GroupServer>(
        "UPDATE group_servers SET \
         priority = COALESCE($1, priority), \
         model_mappings = COALESCE($2, model_mappings), \
         is_enabled = COALESCE($3, is_enabled) \
         WHERE group_id = $4 AND server_id = $5 RETURNING *",
    )
    .bind(input.priority)
    .bind(&input.model_mappings)
    .bind(input.is_enabled)
    .bind(group_id)
    .bind(server_id)
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
    if let Ok(Some(group)) = sqlx::query_as::<_, crate::models::Group>(
        "SELECT * FROM groups WHERE id = $1",
    )
    .bind(group_id)
    .fetch_optional(&state.db)
    .await
    {
        crate::cache::invalidate_group_config(&state.redis, &group.api_key).await;
    }
}
