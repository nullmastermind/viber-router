use axum::{Json, Router, extract::{Query, State}, http::StatusCode, routing::get};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct LogQueryParams {
    pub status_code: Option<i16>,
    pub group_id: Option<Uuid>,
    pub server_id: Option<Uuid>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub api_key: Option<String>,
    pub error_type: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ProxyLogRow {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub group_id: Uuid,
    pub group_api_key: String,
    pub server_id: Uuid,
    pub server_name: String,
    pub request_path: String,
    pub request_method: String,
    pub status_code: i16,
    pub error_type: String,
    pub latency_ms: i32,
    pub failover_chain: serde_json::Value,
    pub request_model: Option<String>,
    pub request_body: Option<serde_json::Value>,
    pub request_headers: Option<serde_json::Value>,
    pub upstream_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LogListResponse {
    pub data: Vec<ProxyLogRow>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct StatusCount {
    pub status_code: i16,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct LogStatsResponse {
    pub total: i64,
    pub by_status: Vec<StatusCount>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_logs))
        .route("/stats", get(log_stats))
}

async fn list_logs(
    State(state): State<AppState>,
    Query(params): Query<LogQueryParams>,
) -> Result<Json<LogListResponse>, (StatusCode, Json<serde_json::Value>)> {
    let page_size = params.page_size.unwrap_or(100).clamp(1, 100);
    let page = params.page.unwrap_or(1).clamp(1, 100);
    let offset = (page - 1) * page_size;

    // Build shared WHERE clause
    let mut conditions = Vec::new();
    let mut param_idx = 0u32;

    if params.status_code.is_some() {
        param_idx += 1;
        conditions.push(format!(
            "(status_code = ${p} OR EXISTS (\
             SELECT 1 FROM jsonb_array_elements(failover_chain) elem \
             WHERE (elem->>'status')::smallint = ${p}))",
            p = param_idx
        ));
    }
    if params.group_id.is_some() {
        param_idx += 1;
        conditions.push(format!("group_id = ${param_idx}"));
    }
    if params.server_id.is_some() {
        param_idx += 1;
        conditions.push(format!("server_id = ${param_idx}"));
    }
    if params.from.is_some() {
        param_idx += 1;
        conditions.push(format!("created_at >= ${param_idx}"));
    }
    if params.to.is_some() {
        param_idx += 1;
        conditions.push(format!("created_at <= ${param_idx}"));
    }
    if params.api_key.is_some() {
        param_idx += 1;
        conditions.push(format!(
            "(group_api_key = ${p} OR EXISTS (\
             SELECT 1 FROM jsonb_array_elements(failover_chain) elem \
             WHERE elem->>'resolved_key' = ${p}))",
            p = param_idx
        ));
    }
    if params.error_type.is_some() {
        param_idx += 1;
        conditions.push(format!("error_type = ${param_idx}"));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" AND {}", conditions.join(" AND "))
    };

    // Data query with OFFSET pagination
    let data_sql = format!(
        "SELECT id, created_at, group_id, group_api_key, server_id, server_name, \
         request_path, request_method, status_code, error_type, latency_ms, \
         failover_chain, request_model, request_body, request_headers, upstream_url \
         FROM proxy_logs WHERE 1=1{where_clause} \
         ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        param_idx + 1,
        param_idx + 2
    );

    // Count query with same filters
    let count_sql = format!("SELECT COUNT(*) FROM proxy_logs WHERE 1=1{where_clause}");

    // Bind helper macro — bind filter params to a query
    macro_rules! bind_filters {
        ($query:expr) => {{
            let mut q = $query;
            if let Some(v) = params.status_code {
                q = q.bind(v);
            }
            if let Some(v) = params.group_id {
                q = q.bind(v);
            }
            if let Some(v) = params.server_id {
                q = q.bind(v);
            }
            if let Some(v) = params.from {
                q = q.bind(v);
            }
            if let Some(v) = params.to {
                q = q.bind(v);
            }
            if let Some(ref v) = params.api_key {
                q = q.bind(v);
            }
            if let Some(ref v) = params.error_type {
                q = q.bind(v);
            }
            q
        }};
    }

    let data_query = bind_filters!(sqlx::query_as::<_, ProxyLogRow>(&data_sql));
    let data_query = data_query.bind(page_size).bind(offset);

    let count_query = bind_filters!(sqlx::query_scalar::<_, i64>(&count_sql));

    let (rows, total) = tokio::try_join!(
        data_query.fetch_all(&state.db),
        count_query.fetch_one(&state.db),
    )
    .map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
    })?;

    Ok(Json(LogListResponse { data: rows, total }))
}

async fn log_stats(
    State(state): State<AppState>,
    Query(params): Query<LogQueryParams>,
) -> Result<Json<LogStatsResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Default to last 24 hours if no date range specified
    let from = params.from.unwrap_or_else(|| Utc::now() - chrono::Duration::hours(24));

    let mut count_sql = String::from("SELECT COUNT(*) FROM proxy_logs WHERE created_at >= $1");
    let mut status_sql = String::from(
        "SELECT status_code, COUNT(*) as count FROM proxy_logs WHERE created_at >= $1",
    );

    let mut param_idx = 1u32;

    let mut extra_conditions = Vec::new();
    if params.to.is_some() {
        param_idx += 1;
        extra_conditions.push(format!("created_at <= ${param_idx}"));
    }
    if params.group_id.is_some() {
        param_idx += 1;
        extra_conditions.push(format!("group_id = ${param_idx}"));
    }
    if params.server_id.is_some() {
        param_idx += 1;
        extra_conditions.push(format!("server_id = ${param_idx}"));
    }
    if params.status_code.is_some() {
        param_idx += 1;
        extra_conditions.push(format!("status_code = ${param_idx}"));
    }
    if params.api_key.is_some() {
        param_idx += 1;
        extra_conditions.push(format!("group_api_key = ${param_idx}"));
    }
    if params.error_type.is_some() {
        param_idx += 1;
        extra_conditions.push(format!("error_type = ${param_idx}"));
    }

    for cond in &extra_conditions {
        count_sql.push_str(" AND ");
        count_sql.push_str(cond);
        status_sql.push_str(" AND ");
        status_sql.push_str(cond);
    }
    status_sql.push_str(" GROUP BY status_code ORDER BY count DESC");

    // Bind params for count query
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql).bind(from);
    let mut status_query = sqlx::query_as::<_, StatusCountRow>(&status_sql).bind(from);

    if let Some(v) = params.to {
        count_query = count_query.bind(v);
        status_query = status_query.bind(v);
    }
    if let Some(v) = params.group_id {
        count_query = count_query.bind(v);
        status_query = status_query.bind(v);
    }
    if let Some(v) = params.server_id {
        count_query = count_query.bind(v);
        status_query = status_query.bind(v);
    }
    if let Some(v) = params.status_code {
        count_query = count_query.bind(v);
        status_query = status_query.bind(v);
    }
    if let Some(ref v) = params.api_key {
        count_query = count_query.bind(v);
        status_query = status_query.bind(v);
    }
    if let Some(ref v) = params.error_type {
        count_query = count_query.bind(v);
        status_query = status_query.bind(v);
    }

    let total = count_query
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
        })?;

    let status_rows = status_query
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
        })?;

    let by_status = status_rows
        .into_iter()
        .map(|r| StatusCount {
            status_code: r.status_code,
            count: r.count,
        })
        .collect();

    Ok(Json(LogStatsResponse { total, by_status }))
}

#[derive(Debug, sqlx::FromRow)]
struct StatusCountRow {
    status_code: i16,
    count: i64,
}
