use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use uuid::Uuid;

use crate::models::{CreateServer, PaginatedResponse, Server, UpdateServer};
use crate::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub search: Option<String>,
}

/// Server fields returned to the client. Credentials are masked
/// when the server has a password_hash and is not in the current session's
/// unlocked set.
#[derive(Debug, Serialize)]
pub struct ServerResponse {
    pub id: Uuid,
    pub short_id: i32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    pub password_hash: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ServerResponse {
    /// Returns real credentials if the server has no password or is
    /// currently unlocked in this session; otherwise credentials are null.
    fn from_server(server: Server, unlocked: &HashSet<Uuid>) -> Self {
        let is_locked = server.password_hash.is_some() && !unlocked.contains(&server.id);
        Self {
            id: server.id,
            short_id: server.short_id,
            name: server.name,
            base_url: if is_locked { None } else { Some(server.base_url) },
            api_key: if is_locked { None } else { server.api_key },
            password_hash: server.password_hash,
            created_at: server.created_at,
            updated_at: server.updated_at,
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_servers).post(create_server))
        .route("/{id}", get(get_server).put(update_server).delete(delete_server))
        .route("/{id}/verify-password", post(verify_password))
}

async fn create_server(
    State(state): State<AppState>,
    Json(input): Json<CreateServer>,
) -> Result<(StatusCode, Json<ServerResponse>), (StatusCode, Json<serde_json::Value>)> {
    let password_hash = input.password.as_ref().map(|pw| {
        let mut hasher = Sha256::new();
        hasher.update(pw.as_bytes());
        format!("{:x}", hasher.finalize())
    });

    let server = sqlx::query_as::<_, Server>(
        "INSERT INTO servers (name, base_url, api_key, password_hash) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(&input.name)
    .bind(&input.base_url)
    .bind(&input.api_key)
    .bind(&password_hash)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
    })?;

    let unlocked = state.unlocked_servers.read().await;
    Ok((StatusCode::CREATED, Json(ServerResponse::from_server(server, &unlocked))))
}

async fn list_servers(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<PaginatedResponse<ServerResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let unlocked = state.unlocked_servers.read().await;

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
    let data: Vec<ServerResponse> = servers.into_iter().map(|s| ServerResponse::from_server(s, &unlocked)).collect();

    Ok(Json(PaginatedResponse { data, total, page, total_pages }))
}

async fn get_server(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ServerResponse>, (StatusCode, Json<serde_json::Value>)> {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Server not found"}))))?;

    let unlocked = state.unlocked_servers.read().await;
    Ok(Json(ServerResponse::from_server(server, &unlocked)))
}

async fn update_server(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateServer>,
) -> Result<Json<ServerResponse>, (StatusCode, Json<serde_json::Value>)> {
    let password_hash = input.password.as_ref().map(|pw_opt| {
        pw_opt.as_ref().map(|pw| {
            let mut hasher = Sha256::new();
            hasher.update(pw.as_bytes());
            format!("{:x}", hasher.finalize())
        })
    });

    let server = match (&input.api_key, &input.password) {
        // None = don't change api_key, None = don't change password
        (None, None) => {
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
        // api_key None + password Some(None) = keep api_key, clear password
        (None, Some(None)) => {
            sqlx::query_as::<_, Server>(
                "UPDATE servers SET \
                 name = COALESCE($1, name), \
                 base_url = COALESCE($2, base_url), \
                 password_hash = NULL, \
                 updated_at = now() \
                 WHERE id = $3 RETURNING *",
            )
            .bind(&input.name)
            .bind(&input.base_url)
            .bind(id)
            .fetch_optional(&state.db)
            .await
        }
        // api_key None + password Some(Some) = keep api_key, set password
        (None, Some(Some(_))) => {
            sqlx::query_as::<_, Server>(
                "UPDATE servers SET \
                 name = COALESCE($1, name), \
                 base_url = COALESCE($2, base_url), \
                 password_hash = $3, \
                 updated_at = now() \
                 WHERE id = $4 RETURNING *",
            )
            .bind(&input.name)
            .bind(&input.base_url)
            .bind(&password_hash)
            .bind(id)
            .fetch_optional(&state.db)
            .await
        }
        // api_key Some(None) = set api_key to NULL, handle password
        (Some(None), None) => {
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
        (Some(None), Some(None)) => {
            sqlx::query_as::<_, Server>(
                "UPDATE servers SET \
                 name = COALESCE($1, name), \
                 base_url = COALESCE($2, base_url), \
                 api_key = NULL, \
                 password_hash = NULL, \
                 updated_at = now() \
                 WHERE id = $3 RETURNING *",
            )
            .bind(&input.name)
            .bind(&input.base_url)
            .bind(id)
            .fetch_optional(&state.db)
            .await
        }
        (Some(None), Some(Some(_))) => {
            sqlx::query_as::<_, Server>(
                "UPDATE servers SET \
                 name = COALESCE($1, name), \
                 base_url = COALESCE($2, base_url), \
                 api_key = NULL, \
                 password_hash = $3, \
                 updated_at = now() \
                 WHERE id = $4 RETURNING *",
            )
            .bind(&input.name)
            .bind(&input.base_url)
            .bind(&password_hash)
            .bind(id)
            .fetch_optional(&state.db)
            .await
        }
        // api_key Some(Some(v)) = set api_key, handle password
        (Some(Some(_)), None) => {
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
            .bind(&input.api_key)
            .bind(id)
            .fetch_optional(&state.db)
            .await
        }
        (Some(Some(v)), Some(None)) => {
            sqlx::query_as::<_, Server>(
                "UPDATE servers SET \
                 name = COALESCE($1, name), \
                 base_url = COALESCE($2, base_url), \
                 api_key = $3, \
                 password_hash = NULL, \
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
        (Some(Some(v)), Some(Some(_))) => {
            sqlx::query_as::<_, Server>(
                "UPDATE servers SET \
                 name = COALESCE($1, name), \
                 base_url = COALESCE($2, base_url), \
                 api_key = $3, \
                 password_hash = $4, \
                 updated_at = now() \
                 WHERE id = $5 RETURNING *",
            )
            .bind(&input.name)
            .bind(&input.base_url)
            .bind(v)
            .bind(&password_hash)
            .bind(id)
            .fetch_optional(&state.db)
            .await
        }
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Server not found"}))))?;

    // Invalidate cache for all groups using this server
    crate::cache::invalidate_groups_by_server(&state.redis, &state.db, id).await;

    let unlocked = state.unlocked_servers.read().await;
    Ok(Json(ServerResponse::from_server(server, &unlocked)))
}

#[derive(Debug, Deserialize)]
struct VerifyPassword {
    password: String,
}

#[derive(Debug, Serialize)]
struct VerifyPasswordResponse {
    base_url: String,
    api_key: Option<String>,
}

async fn verify_password(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<VerifyPassword>,
) -> Result<Json<VerifyPasswordResponse>, (StatusCode, Json<serde_json::Value>)> {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Server not found"}))))?;

    // If no password is set on the server, treat as unlocked
    if let Some(expected_hash) = &server.password_hash {
        let mut hasher = Sha256::new();
        hasher.update(input.password.as_bytes());
        let input_hash = format!("{:x}", hasher.finalize());
        if input_hash != *expected_hash {
            return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Incorrect password"}))));
        }
    }

    // Mark server as unlocked in this session
    state.unlocked_servers.write().await.insert(id);

    Ok(Json(VerifyPasswordResponse {
        base_url: server.base_url,
        api_key: server.api_key,
    }))
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

    // Remove from unlocked set if present
    state.unlocked_servers.write().await.remove(&id);

    Ok(StatusCode::NO_CONTENT)
}
