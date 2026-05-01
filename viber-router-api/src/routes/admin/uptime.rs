use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::Serialize;
use uuid::Uuid;

use crate::routes::AppState;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn internal(e: impl std::fmt::Display) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": e.to_string()})),
    )
}

#[derive(Debug, Serialize)]
pub struct UptimeResponse {
    servers: Vec<ServerUptime>,
}

#[derive(Debug, Serialize)]
pub struct ServerUptime {
    server_id: Uuid,
    server_name: String,
    buckets: Vec<UptimeBucket>,
}

#[derive(Debug, Serialize)]
pub struct UptimeBucket {
    timestamp: i64,
    total: i64,
    success: i64,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_uptime))
}

// PLACEHOLDER_HANDLER

async fn get_uptime(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<UptimeResponse>, ApiError> {
    // Verify group exists
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM groups WHERE id = $1)")
        .bind(group_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;
    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Group not found"})),
        ));
    }

    // Get servers for this group
    let servers = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT gs.server_id, s.name \
         FROM group_servers gs JOIN servers s ON s.id = gs.server_id \
         WHERE gs.group_id = $1 ORDER BY gs.priority",
    )
    .bind(group_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    // Generate 90 bucket timestamps (30-min intervals, covering ~45 hours back from now)
    let now_epoch = chrono::Utc::now().timestamp();
    let bucket_size: i64 = 1800; // 30 minutes
    let current_bucket = (now_epoch / bucket_size) * bucket_size;
    let bucket_timestamps: Vec<i64> = (0..90)
        .rev()
        .map(|i| current_bucket - i * bucket_size)
        .collect();

    // Query aggregated uptime data per server
    #[derive(sqlx::FromRow)]
    struct RawBucket {
        server_id: Uuid,
        bucket: i64,
        total: i64,
        success: i64,
    }

    let cutoff = chrono::DateTime::from_timestamp(bucket_timestamps[0], 0).unwrap_or_default();

    let raw_buckets = sqlx::query_as::<_, RawBucket>(
        "SELECT server_id, \
         (floor(extract(epoch from created_at) / 1800) * 1800)::bigint as bucket, \
         COUNT(*)::bigint as total, \
         COUNT(*) FILTER (WHERE status_code BETWEEN 200 AND 299)::bigint as success \
         FROM uptime_checks \
         WHERE group_id = $1 AND created_at >= $2 \
         GROUP BY server_id, bucket",
    )
    .bind(group_id)
    .bind(cutoff)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    // Build response per server
    let mut result_servers = Vec::with_capacity(servers.len());
    for (server_id, server_name) in &servers {
        let mut buckets = Vec::with_capacity(90);
        for &ts in &bucket_timestamps {
            let matching = raw_buckets
                .iter()
                .find(|b| b.server_id == *server_id && b.bucket == ts);
            buckets.push(UptimeBucket {
                timestamp: ts,
                total: matching.map_or(0, |b| b.total),
                success: matching.map_or(0, |b| b.success),
            });
        }
        result_servers.push(ServerUptime {
            server_id: *server_id,
            server_name: server_name.clone(),
            buckets,
        });
    }

    Ok(Json(UptimeResponse {
        servers: result_servers,
    }))
}
