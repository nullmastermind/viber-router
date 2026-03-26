use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::models::{CreateGroupKey, GroupKey, PaginatedResponse, UpdateGroupKey, generate_api_key};
use crate::routes::AppState;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn err(status: StatusCode, msg: &str) -> ApiError {
    (status, Json(serde_json::json!({"error": msg})))
}

fn internal(e: impl std::fmt::Display) -> ApiError {
    err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub search: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_keys).post(create_key))
        .route("/{key_id}", patch(update_key))
        .route("/{key_id}/regenerate", post(regenerate_key))
        .nest("/{key_id}/allowed-models", super::group_key_allowed_models::router())
        .nest("/{key_id}/subscriptions", super::key_subscriptions::router())
}

async fn create_key(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Json(input): Json<CreateGroupKey>,
) -> Result<(StatusCode, Json<GroupKey>), ApiError> {
    if input.name.len() > 100 {
        return Err(err(StatusCode::BAD_REQUEST, "Name must be 100 characters or less"));
    }

    // Verify group exists
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM groups WHERE id = $1)")
        .bind(group_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

    if !exists {
        return Err(err(StatusCode::NOT_FOUND, "Group not found"));
    }

    let api_key = generate_api_key();
    let key = sqlx::query_as::<_, GroupKey>(
        "INSERT INTO group_keys (group_id, api_key, name) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(group_id)
    .bind(&api_key)
    .bind(&input.name)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    Ok((StatusCode::CREATED, Json(key)))
}


async fn list_keys(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<Json<PaginatedResponse<GroupKey>>, ApiError> {
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let offset = (page - 1) * limit;

    let (total, keys) = if let Some(ref search) = params.search {
        let pattern = format!("%{search}%");
        let (count,) = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM group_keys WHERE group_id = $1 AND (name ILIKE $2 OR api_key ILIKE $2)",
        )
        .bind(group_id)
        .bind(&pattern)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

        let rows = sqlx::query_as::<_, GroupKey>(
            "SELECT * FROM group_keys WHERE group_id = $1 AND (name ILIKE $2 OR api_key ILIKE $2) \
             ORDER BY created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(group_id)
        .bind(&pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(internal)?;

        (count, rows)
    } else {
        let (count,) = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM group_keys WHERE group_id = $1",
        )
        .bind(group_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

        let rows = sqlx::query_as::<_, GroupKey>(
            "SELECT * FROM group_keys WHERE group_id = $1 \
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(group_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(internal)?;

        (count, rows)
    };

    let total_pages = (total as f64 / limit as f64).ceil() as i64;
    Ok(Json(PaginatedResponse { data: keys, total, page, total_pages }))
}


async fn update_key(
    State(state): State<AppState>,
    Path((group_id, key_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateGroupKey>,
) -> Result<Json<GroupKey>, ApiError> {
    if let Some(ref name) = input.name
        && name.len() > 100
    {
        return Err(err(StatusCode::BAD_REQUEST, "Name must be 100 characters or less"));
    }

    let key = sqlx::query_as::<_, GroupKey>(
        "UPDATE group_keys SET \
         name = COALESCE($1, name), \
         is_active = COALESCE($2, is_active), \
         updated_at = now() \
         WHERE id = $3 AND group_id = $4 RETURNING *",
    )
    .bind(&input.name)
    .bind(input.is_active)
    .bind(key_id)
    .bind(group_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Key not found"))?;

    // Invalidate cache for this sub-key
    crate::cache::invalidate_group_config(&state.redis, &key.api_key).await;
    Ok(Json(key))
}

async fn regenerate_key(
    State(state): State<AppState>,
    Path((group_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<GroupKey>, ApiError> {
    let old_key = sqlx::query_as::<_, GroupKey>(
        "SELECT * FROM group_keys WHERE id = $1 AND group_id = $2",
    )
    .bind(key_id)
    .bind(group_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Key not found"))?;

    let new_api_key = generate_api_key();
    let key = sqlx::query_as::<_, GroupKey>(
        "UPDATE group_keys SET api_key = $1, updated_at = now() WHERE id = $2 RETURNING *",
    )
    .bind(&new_api_key)
    .bind(key_id)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    // Invalidate old key's cache
    crate::cache::invalidate_group_config(&state.redis, &old_key.api_key).await;
    Ok(Json(key))
}
