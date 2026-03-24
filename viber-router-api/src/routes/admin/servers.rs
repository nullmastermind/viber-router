use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::models::{CreateServer, PaginatedResponse, Server, UpdateServer};
use crate::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub search: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_servers).post(create_server))
        .route("/{id}", get(get_server).put(update_server).delete(delete_server))
}

async fn create_server(
    State(state): State<AppState>,
    Json(input): Json<CreateServer>,
) -> Result<(StatusCode, Json<Server>), (StatusCode, Json<serde_json::Value>)> {
    let server = sqlx::query_as::<_, Server>(
        "INSERT INTO servers (name, base_url, api_key) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(&input.name)
    .bind(&input.base_url)
    .bind(&input.api_key)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
    })?;

    Ok((StatusCode::CREATED, Json(server)))
}

async fn list_servers(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<PaginatedResponse<Server>>, (StatusCode, Json<serde_json::Value>)> {
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    let (servers, total): (Vec<Server>, i64) = if let Some(search) = &params.search {
        let pattern = format!("%{search}%");
        let servers = sqlx::query_as::<_, Server>(
            "SELECT * FROM servers WHERE name ILIKE $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(&pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM servers WHERE name ILIKE $1")
            .bind(&pattern)
            .fetch_one(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

        (servers, total.0)
    } else {
        let servers = sqlx::query_as::<_, Server>(
            "SELECT * FROM servers ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM servers")
            .fetch_one(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

        (servers, total.0)
    };

    let total_pages = (total as f64 / limit as f64).ceil() as i64;

    Ok(Json(PaginatedResponse { data: servers, total, page, total_pages }))
}

async fn get_server(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Server>, (StatusCode, Json<serde_json::Value>)> {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Server not found"}))))?;

    Ok(Json(server))
}

async fn update_server(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateServer>,
) -> Result<Json<Server>, (StatusCode, Json<serde_json::Value>)> {
    let server = match &input.api_key {
        // None = don't change api_key, use COALESCE to keep current
        None => {
            sqlx::query_as::<_, Server>(
                "UPDATE servers SET \
                 name = COALESCE($1, name), \
                 base_url = COALESCE($2, base_url), \
                 updated_at = now() \
                 WHERE id = $3 RETURNING *",
            )
            .bind(&input.name)
            .bind(&input.base_url)
            .bind(id)
            .fetch_optional(&state.db)
            .await
        }
        // Some(None) = explicitly set api_key to NULL
        Some(None) => {
            sqlx::query_as::<_, Server>(
                "UPDATE servers SET \
                 name = COALESCE($1, name), \
                 base_url = COALESCE($2, base_url), \
                 api_key = NULL, \
                 updated_at = now() \
                 WHERE id = $3 RETURNING *",
            )
            .bind(&input.name)
            .bind(&input.base_url)
            .bind(id)
            .fetch_optional(&state.db)
            .await
        }
        // Some(Some(v)) = set api_key to the provided value
        Some(Some(v)) => {
            sqlx::query_as::<_, Server>(
                "UPDATE servers SET \
                 name = COALESCE($1, name), \
                 base_url = COALESCE($2, base_url), \
                 api_key = $3, \
                 updated_at = now() \
                 WHERE id = $4 RETURNING *",
            )
            .bind(&input.name)
            .bind(&input.base_url)
            .bind(v)
            .bind(id)
            .fetch_optional(&state.db)
            .await
        }
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Server not found"}))))?;

    // Invalidate cache for all groups using this server
    crate::cache::invalidate_groups_by_server(&state.redis, &state.db, id).await;

    Ok(Json(server))
}

async fn delete_server(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    // Check if server is assigned to any groups
    let groups: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT g.id, g.name FROM groups g \
         JOIN group_servers gs ON g.id = gs.group_id \
         WHERE gs.server_id = $1",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    if !groups.is_empty() {
        let group_names: Vec<String> = groups.iter().map(|(_, name)| name.clone()).collect();
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "Server is assigned to groups",
                "groups": group_names
            })),
        ));
    }

    let result = sqlx::query("DELETE FROM servers WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Server not found"}))));
    }

    Ok(StatusCode::NO_CONTENT)
}
