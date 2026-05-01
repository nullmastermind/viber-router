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
        .route(
            "/",
            get(list_group_allowed_models).post(add_group_allowed_model),
        )
        .route(
            "/{model_id}",
            axum::routing::delete(remove_group_allowed_model),
        )
}

async fn list_group_allowed_models(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<Vec<Model>>, ApiError> {
    let models = sqlx::query_as::<_, Model>(
        "SELECT m.* FROM models m \
         JOIN group_allowed_models gam ON m.id = gam.model_id \
         WHERE gam.group_id = $1 ORDER BY m.name",
    )
    .bind(group_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    Ok(Json(models))
}

#[derive(Debug, Deserialize)]
struct AddModelInput {
    model_id: Option<Uuid>,
    name: Option<String>,
}

async fn add_group_allowed_model(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Json(input): Json<AddModelInput>,
) -> Result<(StatusCode, Json<Model>), ApiError> {
    // Resolve model_id: either use provided ID or find/create by name
    let model_id = if let Some(id) = input.model_id {
        id
    } else if let Some(name) = input.name.as_deref() {
        let name = name.trim();
        if name.is_empty() {
            return Err(err(StatusCode::BAD_REQUEST, "Model name is required"));
        }
        // Find or create model by name (upsert to handle concurrent requests)
        let m = sqlx::query_as::<_, Model>(
            "INSERT INTO models (name) VALUES ($1) \
             ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name \
             RETURNING *",
        )
        .bind(name)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;
        m.id
    } else {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "Either model_id or name is required",
        ));
    };

    // Insert into junction table
    sqlx::query("INSERT INTO group_allowed_models (group_id, model_id) VALUES ($1, $2)")
        .bind(group_id)
        .bind(model_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") || e.to_string().contains("unique") {
                err(StatusCode::CONFLICT, "Model already assigned to this group")
            } else {
                internal(e)
            }
        })?;

    // Invalidate cache
    crate::cache::invalidate_group_all_keys(&state.redis, &state.db, group_id).await;

    let model = sqlx::query_as::<_, Model>("SELECT * FROM models WHERE id = $1")
        .bind(model_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

    Ok((StatusCode::CREATED, Json(model)))
}

async fn remove_group_allowed_model(
    State(state): State<AppState>,
    Path((group_id, model_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    // Delete from group_allowed_models (cascade to sub-keys handled manually)
    let result =
        sqlx::query("DELETE FROM group_allowed_models WHERE group_id = $1 AND model_id = $2")
            .bind(group_id)
            .bind(model_id)
            .execute(&state.db)
            .await
            .map_err(internal)?;

    if result.rows_affected() == 0 {
        return Err(err(
            StatusCode::NOT_FOUND,
            "Model not in group's allowed list",
        ));
    }

    // Cascade-delete from all sub-keys of this group
    sqlx::query(
        "DELETE FROM group_key_allowed_models \
         WHERE model_id = $1 AND group_key_id IN (SELECT id FROM group_keys WHERE group_id = $2)",
    )
    .bind(model_id)
    .bind(group_id)
    .execute(&state.db)
    .await
    .map_err(internal)?;

    // Invalidate cache
    crate::cache::invalidate_group_all_keys(&state.redis, &state.db, group_id).await;

    Ok(StatusCode::NO_CONTENT)
}
