use axum::{
    Json,
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::rate_limiter;
use crate::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct UptimeParams {
    key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublicUptimeResponse {
    status: String,
    uptime_percent: f64,
    buckets: Vec<ChainBucket>,
    models: Vec<ModelUptime>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChainBucket {
    timestamp: i64,
    total_requests: i64,
    successful_requests: i64,
}

#[derive(Debug, Serialize)]
pub struct ModelUptime {
    model: String,
    status: String,
    uptime_percent: f64,
    buckets: Vec<ChainBucket>,
}

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({"error": msg})))
}

fn extract_client_ip(headers: &HeaderMap, addr: SocketAddr) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| addr.ip().to_string())
}

fn derive_status(total: i64, successful: i64) -> (String, f64) {
    if total == 0 {
        ("no_data".to_string(), 0.0)
    } else {
        let pct = successful as f64 / total as f64 * 100.0;
        let status = if pct > 95.0 {
            "operational"
        } else if pct >= 50.0 {
            "degraded"
        } else {
            "down"
        };
        (status.to_string(), pct)
    }
}

// PLACEHOLDER_HANDLER

pub async fn public_uptime(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(params): Query<UptimeParams>,
) -> impl IntoResponse {
    let client_ip = extract_client_ip(&headers, addr);

    // Rate limit check
    if rate_limiter::is_ip_rate_limited(&state.redis, &client_ip).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, "60")],
            Json(serde_json::json!({"error": "Too many requests"})),
        )
            .into_response();
    }

    let Some(key) = params.key.filter(|k| !k.is_empty()) else {
        return err(StatusCode::BAD_REQUEST, "key parameter is required").into_response();
    };

    rate_limiter::increment_ip_rate_limit(&state.redis, &client_ip).await;

    // Lookup sub-key + group
    #[derive(sqlx::FromRow)]
    struct KeyInfo {
        group_id: uuid::Uuid,
    }

    let key_info = sqlx::query_as::<_, KeyInfo>(
        "SELECT gk.group_id \
         FROM group_keys gk JOIN groups g ON g.id = gk.group_id \
         WHERE gk.api_key = $1 AND gk.is_active = true",
    )
    .bind(&key)
    .fetch_optional(&state.db)
    .await;

    let key_info = match key_info {
        Ok(Some(info)) => info,
        Ok(None) => {
            return err(StatusCode::FORBIDDEN, "Invalid or inactive key").into_response();
        }
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    // Generate 90 bucket timestamps
    let now_epoch = chrono::Utc::now().timestamp();
    let bucket_size: i64 = 1800;
    let current_bucket = (now_epoch / bucket_size) * bucket_size;
    let bucket_timestamps: Vec<i64> = (0..90).rev().map(|i| current_bucket - i * bucket_size).collect();

    let cutoff = chrono::DateTime::from_timestamp(bucket_timestamps[0], 0)
        .unwrap_or_default();

    // --- All uptime data (overall + per-model) from proxy_logs ---

    // 1. Get allowed model names for this group
    #[derive(sqlx::FromRow)]
    struct ModelName {
        name: String,
    }

    let allowed_models = sqlx::query_as::<_, ModelName>(
        "SELECT m.name \
         FROM models m \
         JOIN group_allowed_models gam ON m.id = gam.model_id \
         WHERE gam.group_id = $1 \
         ORDER BY m.name",
    )
    .bind(key_info.group_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // 2. Query proxy_logs grouped by request_model and 30-min bucket
    #[derive(sqlx::FromRow)]
    struct RawModelBucket {
        request_model: String,
        bucket: i64,
        total_requests: i64,
        successful_requests: i64,
    }

    let raw_model_buckets = sqlx::query_as::<_, RawModelBucket>(
        "SELECT request_model, \
           (floor(extract(epoch from created_at) / 1800) * 1800)::bigint as bucket, \
           COUNT(*)::bigint as total_requests, \
           COUNT(*) FILTER (WHERE status_code BETWEEN 200 AND 299)::bigint as successful_requests \
         FROM proxy_logs \
         WHERE group_id = $1 AND created_at >= $2 AND request_model IS NOT NULL \
         GROUP BY request_model, bucket",
    )
    .bind(key_info.group_id)
    .bind(cutoff)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Index raw model buckets by (model, bucket_ts) for fast lookup
    let mut model_bucket_map: HashMap<(&str, i64), (i64, i64)> = HashMap::new();
    for rb in &raw_model_buckets {
        model_bucket_map.insert(
            (rb.request_model.as_str(), rb.bucket),
            (rb.total_requests, rb.successful_requests),
        );
    }

    // 3. Build per-model entries
    let models: Vec<ModelUptime> = allowed_models
        .iter()
        .map(|m| {
            let model_buckets: Vec<ChainBucket> = bucket_timestamps
                .iter()
                .map(|&ts| {
                    let (total, success) = model_bucket_map
                        .get(&(m.name.as_str(), ts))
                        .copied()
                        .unwrap_or((0, 0));
                    ChainBucket {
                        timestamp: ts,
                        total_requests: total,
                        successful_requests: success,
                    }
                })
                .collect();

            // Use aggregate across all buckets for per-model status
            // (individual models have sparser traffic than the group total,
            // so the last-bucket-only approach often yields "no_data")
            let total_all: i64 = model_buckets.iter().map(|b| b.total_requests).sum();
            let success_all: i64 = model_buckets.iter().map(|b| b.successful_requests).sum();
            let (status, uptime_pct) = derive_status(total_all, success_all);

            ModelUptime {
                model: m.name.clone(),
                status,
                uptime_percent: uptime_pct,
                buckets: model_buckets,
            }
        })
        .collect();

    // 4. Build overall buckets by aggregating all proxy_logs data
    //    (uses the same data source as per-model bars so they are always consistent)
    let mut overall_map: HashMap<i64, (i64, i64)> = HashMap::new();
    for rb in &raw_model_buckets {
        let entry = overall_map.entry(rb.bucket).or_insert((0, 0));
        entry.0 += rb.total_requests;
        entry.1 += rb.successful_requests;
    }
    let buckets: Vec<ChainBucket> = bucket_timestamps
        .iter()
        .map(|&ts| {
            let (total, success) = overall_map.get(&ts).copied().unwrap_or((0, 0));
            ChainBucket {
                timestamp: ts,
                total_requests: total,
                successful_requests: success,
            }
        })
        .collect();

    // 5. Derive overall status from aggregated proxy_logs data
    let overall_total: i64 = buckets.iter().map(|b| b.total_requests).sum();
    let overall_success: i64 = buckets.iter().map(|b| b.successful_requests).sum();
    let (status_text, uptime_percent) = derive_status(overall_total, overall_success);

    Json(PublicUptimeResponse {
        status: status_text,
        uptime_percent,
        buckets,
        models,
    })
    .into_response()
}
