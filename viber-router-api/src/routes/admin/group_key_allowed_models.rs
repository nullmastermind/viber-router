use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::models::Model;
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
        .route("/", get(list_key_allowed_models).post(add_key_allowed_model))
        .route("/{model_id}", axum::routing::delete(remove_key_allowed_model))
}

async fn list_key_allowed_models(
    State(state): State<AppState>,
    Path((_group_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<Model>>, ApiError> {
    let models = sqlx::query_as::<_, Model>(
        "SELECT m.* FROM models m \
         JOIN group_key_allowed_models gkam ON m.id = gkam.model_id \
         WHERE gkam.group_key_id = $1 ORDER BY m.name",
    )
    .bind(key_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    Ok(Json(models))
}

#[derive(Debug, Deserialize)]
struct AddKeyModelInput {
    model_id: Uuid,
}

async fn add_key_allowed_model(
    State(state): State<AppState>,
    Path((group_id, key_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<AddKeyModelInput>,
) -> Result<(StatusCode, Json<Model>), ApiError> {
    // Check that the group has allowed models configured
    let group_model_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM group_allowed_models WHERE group_id = $1",
    )
    .bind(group_id)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    if group_model_count.0 == 0 {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "Group has no allowed models configured. Configure group-level allowed models first.",
        ));
    }

    // Check that the model is in the group's allowed list
    let in_group: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM group_allowed_models WHERE group_id = $1 AND model_id = $2)",
    )
    .bind(group_id)
    .bind(input.model_id)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    if !in_group {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "Model is not in the group's allowed list",
        ));
    }

    // Insert into junction table
    sqlx::query(
        "INSERT INTO group_key_allowed_models (group_key_id, model_id) VALUES ($1, $2)",
    )
    .bind(key_id)
    .bind(input.model_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") || e.to_string().contains("unique") {
            err(StatusCode::CONFLICT, "Model already assigned to this key")
        } else {
            internal(e)
        }
    })?;

    // Invalidate cache
    crate::cache::invalidate_group_all_keys(&state.redis, &state.db, group_id).await;

    let model = sqlx::query_as::<_, Model>("SELECT * FROM models WHERE id = $1")
        .bind(input.model_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

    Ok((StatusCode::CREATED, Json(model)))
}

async fn remove_key_allowed_model(
    State(state): State<AppState>,
    Path((group_id, key_id, model_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query(
        "DELETE FROM group_key_allowed_models WHERE group_key_id = $1 AND model_id = $2",
    )
    .bind(key_id)
    .bind(model_id)
    .execute(&state.db)
    .await
    .map_err(internal)?;

    if result.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Model not in key's allowed list"));
    }

    // Invalidate cache
    crate::cache::invalidate_group_all_keys(&state.redis, &state.db, group_id).await;

    Ok(StatusCode::NO_CONTENT)
}
