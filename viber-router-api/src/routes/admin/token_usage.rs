use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::routes::AppState;
use crate::usage_buffer::hash_key;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn err(status: StatusCode, msg: &str) -> ApiError {
    (status, Json(serde_json::json!({"error": msg})))
}

fn internal(e: impl std::fmt::Display) -> ApiError {
    err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct TokenUsageParams {
    pub group_id: Option<Uuid>,
    pub period: Option<String>,
    pub start: Option<chrono::DateTime<chrono::Utc>>,
    pub end: Option<chrono::DateTime<chrono::Utc>>,
    pub is_dynamic_key: Option<bool>,
    pub key_hash: Option<String>,
}

#[derive(Debug, Serialize)]
struct TokenUsageResponse {
    servers: Vec<ServerTokenUsage>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct ServerTokenUsage {
    server_id: Uuid,
    server_name: String,
    model: Option<String>,
    total_input_tokens: i64,
    total_output_tokens: i64,
    total_cache_creation_tokens: i64,
    total_cache_read_tokens: i64,
    request_count: i64,
}

fn resolve_interval(period: &str) -> &'static str {
    match period {
        "1h" => "1 hour",
        "6h" => "6 hours",
        "24h" => "24 hours",
        "7d" => "7 days",
        "30d" => "30 days",
        _ => "24 hours",
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_token_usage))
}

async fn get_token_usage(
    State(state): State<AppState>,
    Query(params): Query<TokenUsageParams>,
) -> Result<Json<TokenUsageResponse>, ApiError> {
    let group_id = params
        .group_id
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "group_id is required"))?;

    // If the caller passed a raw API key (longer than the 16-char hash we store),
    // hash it so the query matches what's in the database.
    let key_hash = params.key_hash.map(|kh| {
        if kh.len() > 16 { hash_key(&kh) } else { kh }
    });

    let servers = if let (Some(start), Some(end)) = (params.start, params.end) {
        let mut qb = String::from(
            "SELECT t.server_id, s.name as server_name, t.model, \
             COALESCE(SUM(t.input_tokens)::bigint, 0) as total_input_tokens, \
             COALESCE(SUM(t.output_tokens)::bigint, 0) as total_output_tokens, \
             COALESCE(SUM(t.cache_creation_tokens)::bigint, 0) as total_cache_creation_tokens, \
             COALESCE(SUM(t.cache_read_tokens)::bigint, 0) as total_cache_read_tokens, \
             COUNT(*)::bigint as request_count \
             FROM token_usage_logs t JOIN servers s ON s.id = t.server_id \
             WHERE t.group_id = $1 AND t.created_at >= $2 AND t.created_at < $3",
        );
        let mut param_idx = 3;
        if params.is_dynamic_key.is_some() {
            param_idx += 1;
            qb.push_str(&format!(" AND t.is_dynamic_key = ${param_idx}"));
        }
        if key_hash.is_some() {
            param_idx += 1;
            qb.push_str(&format!(" AND t.key_hash = ${param_idx}"));
        }
        qb.push_str(
            " GROUP BY t.server_id, s.name, t.model ORDER BY s.name, t.model",
        );

        let mut query = sqlx::query_as::<_, ServerTokenUsage>(&qb)
            .bind(group_id)
            .bind(start)
            .bind(end);
        if let Some(is_dk) = params.is_dynamic_key {
            query = query.bind(is_dk);
        }
        if let Some(ref kh) = key_hash {
            query = query.bind(kh);
        }
        query.fetch_all(&state.db).await.map_err(internal)?
    } else {
        let interval = resolve_interval(params.period.as_deref().unwrap_or("24h"));

        let mut qb = format!(
            "SELECT t.server_id, s.name as server_name, t.model, \
             COALESCE(SUM(t.input_tokens)::bigint, 0) as total_input_tokens, \
             COALESCE(SUM(t.output_tokens)::bigint, 0) as total_output_tokens, \
             COALESCE(SUM(t.cache_creation_tokens)::bigint, 0) as total_cache_creation_tokens, \
             COALESCE(SUM(t.cache_read_tokens)::bigint, 0) as total_cache_read_tokens, \
             COUNT(*)::bigint as request_count \
             FROM token_usage_logs t JOIN servers s ON s.id = t.server_id \
             WHERE t.group_id = $1 AND t.created_at > now() - interval '{interval}'"
        );
        let mut param_idx = 1;
        if params.is_dynamic_key.is_some() {
            param_idx += 1;
            qb.push_str(&format!(" AND t.is_dynamic_key = ${param_idx}"));
        }
        if key_hash.is_some() {
            param_idx += 1;
            qb.push_str(&format!(" AND t.key_hash = ${param_idx}"));
        }
        qb.push_str(
            " GROUP BY t.server_id, s.name, t.model ORDER BY s.name, t.model",
        );

        let mut query =
            sqlx::query_as::<_, ServerTokenUsage>(&qb).bind(group_id);
        if let Some(is_dk) = params.is_dynamic_key {
            query = query.bind(is_dk);
        }
        if let Some(ref kh) = key_hash {
            query = query.bind(kh);
        }
        query.fetch_all(&state.db).await.map_err(internal)?
    };

    Ok(Json(TokenUsageResponse { servers }))
}
