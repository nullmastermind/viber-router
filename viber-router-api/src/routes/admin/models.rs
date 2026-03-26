use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::models::{CreateModel, Model, PaginatedResponse};
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
        .route("/", get(list_models).post(create_model))
        .route("/{id}", axum::routing::delete(delete_model))
}

async fn list_models(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<PaginatedResponse<Model>>, ApiError> {
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let search_pattern = params.search.as_ref().map(|s| format!("%{s}%"));

    let (total, data) = if let Some(ref pattern) = search_pattern {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM models WHERE name ILIKE $1",
        )
        .bind(pattern)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

        let rows = sqlx::query_as::<_, Model>(
            "SELECT * FROM models WHERE name ILIKE $1 ORDER BY name LIMIT $2 OFFSET $3",
        )
        .bind(pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(internal)?;

        (count, rows)
    } else {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM models")
            .fetch_one(&state.db)
            .await
            .map_err(internal)?;

        let rows = sqlx::query_as::<_, Model>(
            "SELECT * FROM models ORDER BY name LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(internal)?;

        (count, rows)
    };

    let total_pages = (total as f64 / limit as f64).ceil() as i64;
    Ok(Json(PaginatedResponse { data, total, page, total_pages }))
}

async fn create_model(
    State(state): State<AppState>,
    Json(input): Json<CreateModel>,
) -> Result<(StatusCode, Json<Model>), ApiError> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Model name is required"));
    }

    let model = sqlx::query_as::<_, Model>(
        "INSERT INTO models (name) VALUES ($1) RETURNING *",
    )
    .bind(&name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") || e.to_string().contains("unique") {
            err(StatusCode::CONFLICT, "Model already exists")
        } else {
            internal(e)
        }
    })?;

    Ok((StatusCode::CREATED, Json(model)))
}

async fn delete_model(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    // Check if any group uses this model
    let groups: Vec<(String,)> = sqlx::query_as(
        "SELECT g.name FROM groups g \
         JOIN group_allowed_models gam ON g.id = gam.group_id \
         WHERE gam.model_id = $1",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    if !groups.is_empty() {
        let names: Vec<String> = groups.into_iter().map(|(n,)| n).collect();
        return Err(err(
            StatusCode::CONFLICT,
            &format!("Model is in use by groups: {}", names.join(", ")),
        ));
    }

    let result = sqlx::query("DELETE FROM models WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal)?;

    if result.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Model not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}
