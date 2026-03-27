use axum::{
    Json,
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

use crate::rate_limiter;
use crate::routes::AppState;
use crate::subscription;

#[derive(Debug, Deserialize)]
pub struct UsageParams {
    key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublicUsageResponse {
    key_name: String,
    group_name: String,
    api_key: String,
    allowed_models: Vec<String>,
    usage: Vec<ModelUsage>,
    subscriptions: Vec<PublicSubscription>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ModelUsage {
    model: Option<String>,
    total_input_tokens: i64,
    total_output_tokens: i64,
    total_cache_creation_tokens: i64,
    total_cache_read_tokens: i64,
    request_count: i64,
    cost_usd: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct PublicSubscription {
    id: Uuid,
    sub_type: String,
    cost_limit_usd: f64,
    status: String,
    cost_used: f64,
    window_reset_at: Option<String>,
    activated_at: Option<chrono::DateTime<chrono::Utc>>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
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

pub async fn public_usage(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(params): Query<UsageParams>,
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

    // Parse key param
    let Some(key) = params.key.filter(|k| !k.is_empty()) else {
        return err(StatusCode::BAD_REQUEST, "key parameter is required").into_response();
    };

    // Increment rate limit after validating key param is present
    rate_limiter::increment_ip_rate_limit(&state.redis, &client_ip).await;

    // Lookup sub-key + group
    #[derive(sqlx::FromRow)]
    struct KeyInfo {
        key_id: Uuid,
        group_id: Uuid,
        key_name: String,
        group_name: String,
    }

    let key_info = sqlx::query_as::<_, KeyInfo>(
        "SELECT gk.id as key_id, gk.group_id, gk.name as key_name, g.name as group_name \
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

    // Query usage aggregated by model (no server info), last 30 days
    let usage = sqlx::query_as::<_, ModelUsage>(
        "SELECT model, \
         COALESCE(SUM(input_tokens), 0)::bigint as total_input_tokens, \
         COALESCE(SUM(output_tokens), 0)::bigint as total_output_tokens, \
         COALESCE(SUM(cache_creation_tokens), 0)::bigint as total_cache_creation_tokens, \
         COALESCE(SUM(cache_read_tokens), 0)::bigint as total_cache_read_tokens, \
         COUNT(*)::bigint as request_count, \
         SUM(cost_usd) as cost_usd \
         FROM token_usage_logs \
         WHERE group_key_id = $1 AND created_at >= now() - interval '30 days' \
         GROUP BY model ORDER BY model",
    )
    .bind(key_info.key_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Query subscriptions and enrich with cost_used + window_reset_at
    let subs = sqlx::query_as::<_, crate::models::KeySubscription>(
        "SELECT * FROM key_subscriptions WHERE group_key_id = $1 ORDER BY status = 'active' DESC, created_at DESC",
    )
    .bind(key_info.key_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut subscriptions = Vec::with_capacity(subs.len());
    for sub in &subs {
        let cost_used = if sub.status == "active" {
            subscription::get_total_cost(&state, sub).await
        } else {
            0.0
        };
        let window_reset_at = compute_window_reset_at(sub);
        subscriptions.push(PublicSubscription {
            id: sub.id,
            sub_type: sub.sub_type.clone(),
            cost_limit_usd: sub.cost_limit_usd,
            status: sub.status.clone(),
            cost_used,
            window_reset_at,
            activated_at: sub.activated_at,
            expires_at: sub.expires_at,
        });
    }

    // Query allowed models: key-level first, fall back to group-level
    let key_models: Vec<(String,)> = sqlx::query_as(
        "SELECT m.name FROM models m \
         JOIN group_key_allowed_models gkam ON m.id = gkam.model_id \
         WHERE gkam.group_key_id = $1 ORDER BY m.name",
    )
    .bind(key_info.key_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let allowed_models: Vec<String> = if !key_models.is_empty() {
        key_models.into_iter().map(|(n,)| n).collect()
    } else {
        let group_models: Vec<(String,)> = sqlx::query_as(
            "SELECT m.name FROM models m \
             JOIN group_allowed_models gam ON m.id = gam.model_id \
             WHERE gam.group_id = $1 ORDER BY m.name",
        )
        .bind(key_info.group_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        group_models.into_iter().map(|(n,)| n).collect()
    };

    Json(PublicUsageResponse {
        key_name: key_info.key_name,
        group_name: key_info.group_name,
        api_key: key,
        allowed_models,
        usage,
        subscriptions,
    })
    .into_response()
}

fn compute_window_reset_at(sub: &crate::models::KeySubscription) -> Option<String> {
    if sub.sub_type != "hourly_reset" {
        return None;
    }
    let (activated_at, reset_hours) = match (sub.activated_at, sub.reset_hours) {
        (Some(a), Some(r)) => (a, r),
        _ => return None,
    };
    let elapsed = (chrono::Utc::now() - activated_at).num_seconds().max(0) as u64;
    let window_seconds = (reset_hours as u64) * 3600;
    let window_idx = elapsed / window_seconds;
    let window_end = activated_at
        + chrono::Duration::seconds(((window_idx + 1) * window_seconds) as i64);
    Some(window_end.to_rfc3339())
}
