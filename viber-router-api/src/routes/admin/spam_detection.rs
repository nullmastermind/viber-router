use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::PaginatedResponse;
use crate::routes::AppState;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn err(status: StatusCode, msg: &str) -> ApiError {
    (status, Json(serde_json::json!({"error": msg})))
}

fn internal(e: impl std::fmt::Display) -> ApiError {
    err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct SpamDetectionParams {
    pub group_id: Uuid,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SpamResult {
    pub group_key_id: Uuid,
    pub api_key: String,
    pub key_name: String,
    pub spam_type: String,
    pub request_count: i64,
    pub peak_rpm: i64,
    pub detected_at: DateTime<Utc>,
}

struct FlaggedKey {
    group_key_id: Uuid,
    spam_type: String,
    request_count: i64,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_spam_detection))
}

async fn get_spam_detection(
    State(state): State<AppState>,
    Query(params): Query<SpamDetectionParams>,
) -> Result<Json<PaginatedResponse<SpamResult>>, ApiError> {
    let group_id = params.group_id;
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    // Algorithm 1: low-token spam — keys with >= 10 requests with input_tokens < 50 in last 20 min
    let low_token_rows = sqlx::query_as::<_, (Uuid, i64)>(
        "SELECT group_key_id, COUNT(*)::bigint as request_count \
         FROM token_usage_logs \
         WHERE group_id = $1 \
           AND group_key_id IS NOT NULL \
           AND input_tokens < 50 \
           AND created_at > now() - interval '20 minutes' \
         GROUP BY group_key_id \
         HAVING COUNT(*) >= 10",
    )
    .bind(group_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    // Algorithm 2: duplicate-request spam — keys with >= 10 requests with same content_hash in last 10 min
    let duplicate_rows = sqlx::query_as::<_, (Uuid, i64)>(
        "SELECT group_key_id, MAX(cnt)::bigint as request_count \
         FROM ( \
           SELECT group_key_id, content_hash, COUNT(*)::bigint as cnt \
           FROM token_usage_logs \
           WHERE group_id = $1 \
             AND group_key_id IS NOT NULL \
             AND content_hash IS NOT NULL \
             AND created_at > now() - interval '10 minutes' \
           GROUP BY group_key_id, content_hash \
           HAVING COUNT(*) >= 10 \
         ) sub \
         GROUP BY group_key_id",
    )
    .bind(group_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    // Merge results
    let mut flagged: Vec<FlaggedKey> = Vec::new();
    for (key_id, count) in low_token_rows {
        flagged.push(FlaggedKey {
            group_key_id: key_id,
            spam_type: "low_token".to_string(),
            request_count: count,
        });
    }
    for (key_id, count) in duplicate_rows {
        flagged.push(FlaggedKey {
            group_key_id: key_id,
            spam_type: "duplicate_request".to_string(),
            request_count: count,
        });
    }

    let total = flagged.len() as i64;
    let total_pages = if limit > 0 {
        (total as f64 / limit as f64).ceil() as i64
    } else {
        0
    };

    // Apply pagination
    let page_items: Vec<&FlaggedKey> = flagged
        .iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    let detected_at = Utc::now();
    let mut results: Vec<SpamResult> = Vec::with_capacity(page_items.len());

    for item in page_items {
        // Fetch api_key and name from group_keys
        let key_info = sqlx::query_as::<_, (String, String)>(
            "SELECT api_key, name FROM group_keys WHERE id = $1",
        )
        .bind(item.group_key_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?;

        let (api_key, key_name) = match key_info {
            Some(info) => info,
            None => continue,
        };

        // Compute peak_rpm using date_trunc('minute', created_at) within the detection window
        let window_interval = if item.spam_type == "low_token" {
            "20 minutes"
        } else {
            "10 minutes"
        };
        let peak_rpm: i64 = sqlx::query_scalar::<_, i64>(&format!(
            "SELECT COALESCE(MAX(cnt), 0)::bigint FROM ( \
               SELECT COUNT(*)::bigint as cnt \
               FROM token_usage_logs \
               WHERE group_key_id = $1 \
                 AND created_at > now() - interval '{window_interval}' \
               GROUP BY date_trunc('minute', created_at) \
             ) sub",
        ))
        .bind(item.group_key_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

        results.push(SpamResult {
            group_key_id: item.group_key_id,
            api_key,
            key_name,
            spam_type: item.spam_type.clone(),
            request_count: item.request_count,
            peak_rpm,
            detected_at,
        });
    }

    Ok(Json(PaginatedResponse {
        data: results,
        total,
        page,
        total_pages,
    }))
}
