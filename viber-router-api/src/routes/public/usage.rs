use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::TimeZone;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::routes::AppState;
use crate::subscription;

#[derive(Debug, Deserialize)]
pub struct UsageParams {
    key: Option<String>,
    period: Option<String>,
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
    effective_input_tokens: i64,
    total_output_tokens: i64,
    request_count: i64,
    cost_usd: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct QuotaInfo {
    name: String,
    utilization: f64,
    reset_at: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BonusModelUsage {
    model: String,
    request_count: i64,
    cost_usd: f64,
}

#[derive(Debug, Serialize)]
pub struct PublicSubscription {
    id: Uuid,
    sub_type: String,
    cost_limit_usd: f64,
    rpm_limit: Option<f64>,
    tpm_limit: Option<f64>,
    status: String,
    cost_used: f64,
    window_reset_at: Option<String>,
    activated_at: Option<chrono::DateTime<chrono::Utc>>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    bonus_name: Option<String>,
    bonus_quotas: Option<Vec<QuotaInfo>>,
    bonus_usage: Option<Vec<BonusModelUsage>>,
}

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({"error": msg})))
}

pub async fn public_usage(
    State(state): State<AppState>,
    Query(params): Query<UsageParams>,
) -> impl IntoResponse {
    // Parse key param
    let Some(key) = params.key.filter(|k| !k.is_empty()) else {
        return err(StatusCode::BAD_REQUEST, "key parameter is required").into_response();
    };

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

    // Map period param to a SQL interval (allowlist — never interpolate user input directly)
    let interval = match params.period.as_deref() {
        Some("1h") => "1 hour",
        Some("6h") => "6 hours",
        Some("24h") => "24 hours",
        Some("7d") => "7 days",
        _ => "30 days",
    };

    // Query usage aggregated by model (no server info), for the selected period
    let usage = sqlx::query_as::<_, ModelUsage>(
        &format!(
            "SELECT model, \
             (COALESCE(SUM(input_tokens), 0) + COALESCE(SUM(cache_creation_tokens), 0) + COALESCE(SUM(cache_read_tokens), 0))::bigint as effective_input_tokens, \
             COALESCE(SUM(output_tokens), 0)::bigint as total_output_tokens, \
             COUNT(*)::bigint as request_count, \
             SUM(cost_usd) as cost_usd \
             FROM token_usage_logs \
             WHERE group_key_id = $1 AND created_at >= now() - interval '{interval}' \
             GROUP BY model ORDER BY model"
        ),
    )
    .bind(key_info.key_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Query subscriptions and enrich with cost_used + window_reset_at
    let subs = sqlx::query_as::<_, crate::models::KeySubscription>(
        "SELECT * FROM key_subscriptions WHERE group_key_id = $1 AND status != 'cancelled' ORDER BY status = 'active' DESC, created_at DESC",
    )
    .bind(key_info.key_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut subscriptions = Vec::with_capacity(subs.len());
    for sub in &subs {
        let is_bonus = sub.sub_type == "bonus";

        let cost_used = if sub.status == "active" && !is_bonus {
            subscription::get_total_cost(&state, sub).await
        } else {
            0.0
        };
        let window_reset_at = if !is_bonus {
            compute_window_reset_at(&state, sub).await
        } else {
            None
        };

        let (bonus_quotas, bonus_usage) = if is_bonus {
            // Fetch per-model request counts for last 30 days
            let usage_rows: Vec<(Option<String>, i64, f64)> = sqlx::query_as(
                "SELECT model, COUNT(*)::bigint as request_count, COALESCE(SUM(cost_usd), 0)::float8 as cost_usd \
                 FROM token_usage_logs \
                 WHERE subscription_id = $1 AND created_at >= now() - interval '30 days' \
                 GROUP BY model ORDER BY model",
            )
            .bind(sub.id)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

            let bonus_usage_data: Vec<BonusModelUsage> = usage_rows
                .into_iter()
                .filter_map(|(model, count, cost)| {
                    model.map(|m| BonusModelUsage {
                        model: m,
                        request_count: count,
                        cost_usd: cost,
                    })
                })
                .collect();

            // Fetch quota data if bonus_quota_url is set
            // None → null in JSON (no URL configured)
            // Some([]) → empty array (fetch failed)
            // Some([...]) → populated (fetch succeeded)
            let quotas: Option<Vec<QuotaInfo>> = if let Some(ref quota_url) = sub.bonus_quota_url {
                fetch_bonus_quotas(&state, quota_url, sub.bonus_quota_headers.as_ref()).await
            } else {
                None
            };

            (quotas, Some(bonus_usage_data))
        } else {
            (None, None)
        };

        subscriptions.push(PublicSubscription {
            id: sub.id,
            sub_type: sub.sub_type.clone(),
            cost_limit_usd: sub.cost_limit_usd,
            rpm_limit: sub.rpm_limit,
            tpm_limit: sub.tpm_limit,
            status: sub.status.clone(),
            cost_used,
            window_reset_at,
            activated_at: sub.activated_at,
            expires_at: sub.expires_at,
            bonus_name: sub.bonus_name.clone(),
            bonus_quotas,
            bonus_usage,
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

async fn compute_window_reset_at(
    state: &AppState,
    sub: &crate::models::KeySubscription,
) -> Option<String> {
    let reset_hours = sub.reset_hours?;

    // Read the active window start from Redis
    use deadpool_redis::redis::AsyncCommands;
    let key = format!("sub_window_start:{}", sub.id);
    let mut conn = state.redis.get().await.ok()?;
    let val: Option<String> = conn.get(&key).await.ok()?;
    let ws: i64 = val?.parse().ok()?;

    let ws_dt = chrono::Utc.timestamp_opt(ws, 0).single()?;
    let reset_at = ws_dt + chrono::Duration::seconds((reset_hours as i64) * 3600);
    Some(reset_at.to_rfc3339())
}

/// Fetch quota data from a bonus_quota_url with a 5-second timeout.
/// Returns None only if the quota URL was not set or request fails.
/// Returns Some(vec) — empty vec on error, populated vec on success.
async fn fetch_bonus_quotas(
    state: &AppState,
    quota_url: &str,
    quota_headers: Option<&serde_json::Value>,
) -> Option<Vec<QuotaInfo>> {
    let mut req = state
        .http_client
        .get(quota_url)
        .timeout(std::time::Duration::from_secs(5));

    // Apply custom headers if provided
    if let Some(headers_val) = quota_headers
        && let Some(obj) = headers_val.as_object()
    {
        for (key, val) in obj {
            if let Some(val_str) = val.as_str()
                && let (Ok(header_name), Ok(header_val)) = (
                    reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                    reqwest::header::HeaderValue::from_str(val_str),
                )
            {
                req = req.header(header_name, header_val);
            }
        }
    }

    let resp = match req.send().await {
        Ok(r) if r.status().is_success() => r,
        Ok(_) => return Some(vec![]),
        Err(_) => return Some(vec![]),
    };

    let body: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return Some(vec![]),
    };

    let quotas_arr = match body.get("quotas").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Some(vec![]),
    };

    let quotas: Vec<QuotaInfo> = quotas_arr
        .iter()
        .filter_map(|item| {
            let name = item.get("name")?.as_str()?.to_string();
            let utilization = item.get("utilization")?.as_f64()?;
            let reset_at = item
                .get("reset_at")
                .and_then(|v| v.as_str())
                .map(String::from);
            let description = item
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from);
            Some(QuotaInfo {
                name,
                utilization,
                reset_at,
                description,
            })
        })
        .collect();

    Some(quotas)
}
