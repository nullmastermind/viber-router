use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::routes::AppState;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn err(status: StatusCode, msg: &str) -> ApiError {
    (status, Json(serde_json::json!({"error": msg})))
}

fn internal(e: impl std::fmt::Display) -> ApiError {
    err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct TtftStatsParams {
    pub group_id: Option<Uuid>,
    pub period: Option<String>,
}

#[derive(Debug, Serialize)]
struct TtftStatsResponse {
    servers: Vec<ServerTtftStats>,
}

#[derive(Debug, Serialize)]
struct ServerTtftStats {
    server_id: Uuid,
    server_name: String,
    avg_ttft_ms: Option<f64>,
    p50_ttft_ms: Option<f64>,
    p95_ttft_ms: Option<f64>,
    timeout_count: i64,
    total_count: i64,
    data_points: Vec<TtftDataPointOut>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct TtftDataPoint {
    server_id: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    ttft_ms: Option<i32>,
    timed_out: bool,
}

#[derive(Debug, Serialize)]
struct TtftDataPointOut {
    created_at: chrono::DateTime<chrono::Utc>,
    ttft_ms: Option<i32>,
    timed_out: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct AggRow {
    server_id: Uuid,
    server_name: String,
    avg_ttft_ms: Option<f64>,
    p50_ttft_ms: Option<f64>,
    p95_ttft_ms: Option<f64>,
    timeout_count: i64,
    total_count: i64,
}

/// Validated interval values — only these strings can appear in SQL.
fn resolve_interval(period: &str) -> &'static str {
    match period {
        "1h" => "1 hour",
        "6h" => "6 hours",
        "24h" => "24 hours",
        _ => "1 hour",
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_ttft_stats))
}

async fn get_ttft_stats(
    State(state): State<AppState>,
    Query(params): Query<TtftStatsParams>,
) -> Result<Json<TtftStatsResponse>, ApiError> {
    let group_id = params.group_id
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "group_id is required"))?;

    let interval = resolve_interval(params.period.as_deref().unwrap_or("1h"));

    // Single query for aggregated stats
    let agg_rows = sqlx::query_as::<_, AggRow>(&format!(
        "SELECT t.server_id, s.name as server_name, \
         AVG(t.ttft_ms)::float8 as avg_ttft_ms, \
         PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY t.ttft_ms)::float8 as p50_ttft_ms, \
         PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY t.ttft_ms)::float8 as p95_ttft_ms, \
         COUNT(*) FILTER (WHERE t.timed_out) as timeout_count, \
         COUNT(*) as total_count \
         FROM ttft_logs t JOIN servers s ON s.id = t.server_id \
         WHERE t.group_id = $1 AND t.created_at > now() - interval '{interval}' \
         GROUP BY t.server_id, s.name \
         ORDER BY s.name"
    ))
    .bind(group_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    // Single query for all data points (no N+1)
    let all_points = sqlx::query_as::<_, TtftDataPoint>(&format!(
        "SELECT t.server_id, t.created_at, t.ttft_ms, t.timed_out FROM ttft_logs t \
         WHERE t.group_id = $1 AND t.created_at > now() - interval '{interval}' \
         ORDER BY t.created_at"
    ))
    .bind(group_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    // Group data points by server_id
    let mut points_by_server: std::collections::HashMap<Uuid, Vec<TtftDataPointOut>> =
        std::collections::HashMap::new();
    for p in all_points {
        points_by_server.entry(p.server_id).or_default().push(TtftDataPointOut {
            created_at: p.created_at,
            ttft_ms: p.ttft_ms,
            timed_out: p.timed_out,
        });
    }

    let servers = agg_rows
        .into_iter()
        .map(|row| {
            let data_points = points_by_server.remove(&row.server_id).unwrap_or_default();
            ServerTtftStats {
                server_id: row.server_id,
                server_name: row.server_name,
                avg_ttft_ms: row.avg_ttft_ms,
                p50_ttft_ms: row.p50_ttft_ms,
                p95_ttft_ms: row.p95_ttft_ms,
                timeout_count: row.timeout_count,
                total_count: row.total_count,
                data_points,
            }
        })
        .collect();

    Ok(Json(TtftStatsResponse { servers }))
}
