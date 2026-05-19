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
    pub group_key_id: Option<String>,
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
    cost_usd: Option<f64>,
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

#[derive(Debug, Serialize, sqlx::FromRow)]
struct KeyTokenUsage {
    group_key_id: Option<Uuid>,
    key_name: Option<String>,
    api_key: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    total_input_tokens: i64,
    total_output_tokens: i64,
    total_cache_creation_tokens: i64,
    total_cache_read_tokens: i64,
    request_count: i64,
    cost_usd: Option<f64>,
    peak_tpm: i64,
}

#[derive(Debug, Serialize)]
struct KeyUsageResponse {
    keys: Vec<KeyTokenUsage>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_token_usage))
        .route("/by-key", get(get_token_usage_by_key))
}

enum GroupKeyIdFilter {
    IsNull,
    Equals(Uuid),
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
    let key_hash = params
        .key_hash
        .map(|kh| if kh.len() > 16 { hash_key(&kh) } else { kh });

    // group_key_id filter: "null" means master key (IS NULL), UUID means specific sub-key
    let group_key_id_filter: Option<GroupKeyIdFilter> = match params.group_key_id {
        None => None,
        Some(ref v) if v == "null" => Some(GroupKeyIdFilter::IsNull),
        Some(ref v) => {
            let id = v
                .parse::<Uuid>()
                .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid group_key_id"))?;
            Some(GroupKeyIdFilter::Equals(id))
        }
    };

    let servers = if let (Some(start), Some(end)) = (params.start, params.end) {
        let mut qb = String::from(
            "SELECT t.server_id, s.name as server_name, t.model, \
             COALESCE(SUM(t.input_tokens)::bigint, 0) as total_input_tokens, \
             COALESCE(SUM(t.output_tokens)::bigint, 0) as total_output_tokens, \
             COALESCE(SUM(t.cache_creation_tokens)::bigint, 0) as total_cache_creation_tokens, \
             COALESCE(SUM(t.cache_read_tokens)::bigint, 0) as total_cache_read_tokens, \
             COUNT(*)::bigint as request_count, \
             CASE WHEN m.id IS NOT NULL AND (m.input_1m_usd IS NOT NULL OR m.output_1m_usd IS NOT NULL OR m.cache_write_1m_usd IS NOT NULL OR m.cache_read_1m_usd IS NOT NULL) THEN \
               (COALESCE(SUM(t.input_tokens::float8 * COALESCE(m.input_1m_usd, 0) * COALESCE(gs.rate_input, 1.0)), 0) + \
                COALESCE(SUM(t.output_tokens::float8 * COALESCE(m.output_1m_usd, 0) * COALESCE(gs.rate_output, 1.0)), 0) + \
                COALESCE(SUM(t.cache_creation_tokens::float8 * COALESCE(m.cache_write_1m_usd, 0) * COALESCE(gs.rate_cache_write, 1.0)), 0) + \
                COALESCE(SUM(t.cache_read_tokens::float8 * COALESCE(m.cache_read_1m_usd, 0) * COALESCE(gs.rate_cache_read, 1.0)), 0)) / 1000000.0 \
             ELSE NULL END as cost_usd \
             FROM token_usage_logs t \
             JOIN servers s ON s.id = t.server_id \
             LEFT JOIN models m ON m.name = t.model \
             LEFT JOIN group_servers gs ON gs.group_id = t.group_id AND gs.server_id = t.server_id \
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
        match &group_key_id_filter {
            Some(GroupKeyIdFilter::IsNull) => {
                qb.push_str(" AND t.group_key_id IS NULL");
            }
            Some(GroupKeyIdFilter::Equals(_)) => {
                param_idx += 1;
                qb.push_str(&format!(" AND t.group_key_id = ${param_idx}"));
            }
            None => {}
        }
        qb.push_str(
            " GROUP BY t.server_id, s.name, t.model, m.id, m.input_1m_usd, m.output_1m_usd, m.cache_write_1m_usd, m.cache_read_1m_usd, \
             gs.rate_input, gs.rate_output, gs.rate_cache_write, gs.rate_cache_read \
             ORDER BY s.name, t.model",
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
        if let Some(GroupKeyIdFilter::Equals(id)) = &group_key_id_filter {
            query = query.bind(id);
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
             COUNT(*)::bigint as request_count, \
             CASE WHEN m.id IS NOT NULL AND (m.input_1m_usd IS NOT NULL OR m.output_1m_usd IS NOT NULL OR m.cache_write_1m_usd IS NOT NULL OR m.cache_read_1m_usd IS NOT NULL) THEN \
               (COALESCE(SUM(t.input_tokens::float8 * COALESCE(m.input_1m_usd, 0) * COALESCE(gs.rate_input, 1.0)), 0) + \
                COALESCE(SUM(t.output_tokens::float8 * COALESCE(m.output_1m_usd, 0) * COALESCE(gs.rate_output, 1.0)), 0) + \
                COALESCE(SUM(t.cache_creation_tokens::float8 * COALESCE(m.cache_write_1m_usd, 0) * COALESCE(gs.rate_cache_write, 1.0)), 0) + \
                COALESCE(SUM(t.cache_read_tokens::float8 * COALESCE(m.cache_read_1m_usd, 0) * COALESCE(gs.rate_cache_read, 1.0)), 0)) / 1000000.0 \
             ELSE NULL END as cost_usd \
             FROM token_usage_logs t \
             JOIN servers s ON s.id = t.server_id \
             LEFT JOIN models m ON m.name = t.model \
             LEFT JOIN group_servers gs ON gs.group_id = t.group_id AND gs.server_id = t.server_id \
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
        match &group_key_id_filter {
            Some(GroupKeyIdFilter::IsNull) => {
                qb.push_str(" AND t.group_key_id IS NULL");
            }
            Some(GroupKeyIdFilter::Equals(_)) => {
                param_idx += 1;
                qb.push_str(&format!(" AND t.group_key_id = ${param_idx}"));
            }
            None => {}
        }
        qb.push_str(
            " GROUP BY t.server_id, s.name, t.model, m.id, m.input_1m_usd, m.output_1m_usd, m.cache_write_1m_usd, m.cache_read_1m_usd, \
             gs.rate_input, gs.rate_output, gs.rate_cache_write, gs.rate_cache_read \
             ORDER BY s.name, t.model",
        );

        let mut query = sqlx::query_as::<_, ServerTokenUsage>(&qb).bind(group_id);
        if let Some(is_dk) = params.is_dynamic_key {
            query = query.bind(is_dk);
        }
        if let Some(ref kh) = key_hash {
            query = query.bind(kh);
        }
        if let Some(GroupKeyIdFilter::Equals(id)) = &group_key_id_filter {
            query = query.bind(id);
        }
        query.fetch_all(&state.db).await.map_err(internal)?
    };

    Ok(Json(TokenUsageResponse { servers }))
}

async fn get_token_usage_by_key(
    State(state): State<AppState>,
    Query(params): Query<TokenUsageParams>,
) -> Result<Json<KeyUsageResponse>, ApiError> {
    let group_id = params
        .group_id
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "group_id is required"))?;

    // Peak TPM is computed in a separate, concurrent query because folding it
    // into the main aggregation joined plan dramatically increases scan cost
    // on token_usage_logs (observed >9s on real data).
    #[derive(sqlx::FromRow)]
    struct PeakRow {
        group_key_id: Option<Uuid>,
        peak_tpm: i64,
    }

    let (keys, peak_rows) = if let (Some(start), Some(end)) = (params.start, params.end) {
        let main_sql = String::from(
            "SELECT t.group_key_id, gk.name as key_name, gk.api_key, gk.created_at, \
             COALESCE(SUM(t.input_tokens)::bigint, 0) as total_input_tokens, \
             COALESCE(SUM(t.output_tokens)::bigint, 0) as total_output_tokens, \
             COALESCE(SUM(t.cache_creation_tokens)::bigint, 0) as total_cache_creation_tokens, \
             COALESCE(SUM(t.cache_read_tokens)::bigint, 0) as total_cache_read_tokens, \
             COUNT(*)::bigint as request_count, \
             CASE WHEN bool_or(m.id IS NOT NULL AND (m.input_1m_usd IS NOT NULL OR m.output_1m_usd IS NOT NULL OR m.cache_write_1m_usd IS NOT NULL OR m.cache_read_1m_usd IS NOT NULL)) THEN \
               (COALESCE(SUM(t.input_tokens::float8 * COALESCE(m.input_1m_usd, 0) * COALESCE(gs.rate_input, 1.0)), 0) + \
                COALESCE(SUM(t.output_tokens::float8 * COALESCE(m.output_1m_usd, 0) * COALESCE(gs.rate_output, 1.0)), 0) + \
                COALESCE(SUM(t.cache_creation_tokens::float8 * COALESCE(m.cache_write_1m_usd, 0) * COALESCE(gs.rate_cache_write, 1.0)), 0) + \
                COALESCE(SUM(t.cache_read_tokens::float8 * COALESCE(m.cache_read_1m_usd, 0) * COALESCE(gs.rate_cache_read, 1.0)), 0)) / 1000000.0 \
             ELSE NULL END as cost_usd, \
             0::bigint as peak_tpm \
             FROM token_usage_logs t \
             LEFT JOIN group_keys gk ON gk.id = t.group_key_id \
             LEFT JOIN models m ON m.name = t.model \
             LEFT JOIN group_servers gs ON gs.group_id = t.group_id AND gs.server_id = t.server_id \
             WHERE t.group_id = $1 AND t.created_at >= $2 AND t.created_at < $3 \
             GROUP BY t.group_key_id, gk.name, gk.api_key, gk.created_at \
             ORDER BY request_count DESC",
        );
        let peak_sql = String::from(
            "SELECT group_key_id, COALESCE(MAX(tpm), 0)::bigint AS peak_tpm FROM ( \
               SELECT group_key_id, \
                 COALESCE(SUM(COALESCE(input_tokens, 0) + COALESCE(output_tokens, 0) + COALESCE(cache_creation_tokens, 0) + COALESCE(cache_read_tokens, 0)), 0) AS tpm \
               FROM token_usage_logs \
               WHERE group_id = $1 AND created_at >= $2 AND created_at < $3 \
               GROUP BY group_key_id, date_trunc('minute', created_at) \
             ) pm GROUP BY group_key_id",
        );

        let main_fut = sqlx::query_as::<_, KeyTokenUsage>(&main_sql)
            .bind(group_id)
            .bind(start)
            .bind(end)
            .fetch_all(&state.db);
        let peak_fut = sqlx::query_as::<_, PeakRow>(&peak_sql)
            .bind(group_id)
            .bind(start)
            .bind(end)
            .fetch_all(&state.db);
        tokio::try_join!(main_fut, peak_fut).map_err(internal)?
    } else {
        let interval = resolve_interval(params.period.as_deref().unwrap_or("24h"));

        let main_sql = format!(
            "SELECT t.group_key_id, gk.name as key_name, gk.api_key, gk.created_at, \
             COALESCE(SUM(t.input_tokens)::bigint, 0) as total_input_tokens, \
             COALESCE(SUM(t.output_tokens)::bigint, 0) as total_output_tokens, \
             COALESCE(SUM(t.cache_creation_tokens)::bigint, 0) as total_cache_creation_tokens, \
             COALESCE(SUM(t.cache_read_tokens)::bigint, 0) as total_cache_read_tokens, \
             COUNT(*)::bigint as request_count, \
             CASE WHEN bool_or(m.id IS NOT NULL AND (m.input_1m_usd IS NOT NULL OR m.output_1m_usd IS NOT NULL OR m.cache_write_1m_usd IS NOT NULL OR m.cache_read_1m_usd IS NOT NULL)) THEN \
               (COALESCE(SUM(t.input_tokens::float8 * COALESCE(m.input_1m_usd, 0) * COALESCE(gs.rate_input, 1.0)), 0) + \
                COALESCE(SUM(t.output_tokens::float8 * COALESCE(m.output_1m_usd, 0) * COALESCE(gs.rate_output, 1.0)), 0) + \
                COALESCE(SUM(t.cache_creation_tokens::float8 * COALESCE(m.cache_write_1m_usd, 0) * COALESCE(gs.rate_cache_write, 1.0)), 0) + \
                COALESCE(SUM(t.cache_read_tokens::float8 * COALESCE(m.cache_read_1m_usd, 0) * COALESCE(gs.rate_cache_read, 1.0)), 0)) / 1000000.0 \
             ELSE NULL END as cost_usd, \
             0::bigint as peak_tpm \
             FROM token_usage_logs t \
             LEFT JOIN group_keys gk ON gk.id = t.group_key_id \
             LEFT JOIN models m ON m.name = t.model \
             LEFT JOIN group_servers gs ON gs.group_id = t.group_id AND gs.server_id = t.server_id \
             WHERE t.group_id = $1 AND t.created_at > now() - interval '{interval}' \
             GROUP BY t.group_key_id, gk.name, gk.api_key, gk.created_at \
             ORDER BY request_count DESC"
        );
        let peak_sql = format!(
            "SELECT group_key_id, COALESCE(MAX(tpm), 0)::bigint AS peak_tpm FROM ( \
               SELECT group_key_id, \
                 COALESCE(SUM(COALESCE(input_tokens, 0) + COALESCE(output_tokens, 0) + COALESCE(cache_creation_tokens, 0) + COALESCE(cache_read_tokens, 0)), 0) AS tpm \
               FROM token_usage_logs \
               WHERE group_id = $1 AND created_at > now() - interval '{interval}' \
               GROUP BY group_key_id, date_trunc('minute', created_at) \
             ) pm GROUP BY group_key_id"
        );

        let main_fut = sqlx::query_as::<_, KeyTokenUsage>(&main_sql)
            .bind(group_id)
            .fetch_all(&state.db);
        let peak_fut = sqlx::query_as::<_, PeakRow>(&peak_sql)
            .bind(group_id)
            .fetch_all(&state.db);
        tokio::try_join!(main_fut, peak_fut).map_err(internal)?
    };

    let peak_map: std::collections::HashMap<Option<Uuid>, i64> = peak_rows
        .into_iter()
        .map(|r| (r.group_key_id, r.peak_tpm))
        .collect();
    let mut keys = keys;
    for k in &mut keys {
        k.peak_tpm = peak_map.get(&k.group_key_id).copied().unwrap_or(0);
    }

    Ok(Json(KeyUsageResponse { keys }))
}

/// Pure-logic mirror of the `peak` CTE used in `get_token_usage_by_key`.
///
/// For each `group_key_id`, returns the maximum sum of total tokens
/// (input + output + cache_creation + cache_read) observed in any single
/// 1-minute bucket (`date_trunc('minute', created_at)`).
///
/// Kept as a standalone helper so the bucketing rule can be unit-tested
/// without a Postgres harness.
#[cfg(test)]
pub fn compute_peak_tpm(
    rows: &[(Option<Uuid>, chrono::DateTime<chrono::Utc>, i64)],
) -> std::collections::HashMap<Option<Uuid>, i64> {
    use chrono::Timelike;
    use std::collections::HashMap;

    let mut per_minute: HashMap<(Option<Uuid>, chrono::DateTime<chrono::Utc>), i64> =
        HashMap::new();
    for (key, ts, tokens) in rows {
        let minute = ts
            .with_second(0)
            .and_then(|t| t.with_nanosecond(0))
            .unwrap_or(*ts);
        *per_minute.entry((*key, minute)).or_insert(0) += *tokens;
    }
    let mut peak: HashMap<Option<Uuid>, i64> = HashMap::new();
    for ((key, _), tpm) in per_minute {
        let entry = peak.entry(key).or_insert(0);
        if tpm > *entry {
            *entry = tpm;
        }
    }
    peak
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn peak_tpm_picks_max_minute_per_key() {
        let k1 = Some(Uuid::from_u128(1));
        let k2 = Some(Uuid::from_u128(2));
        let t = |min: u32, sec: u32| Utc.with_ymd_and_hms(2026, 5, 19, 10, min, sec).unwrap();

        let rows = vec![
            // k1, minute 10 -> 100 + 200 = 300 (peak)
            (k1, t(10, 5), 100),
            (k1, t(10, 45), 200),
            // k1, minute 11 -> 50
            (k1, t(11, 0), 50),
            // k2, minute 10 -> 10
            (k2, t(10, 30), 10),
            // k2, minute 12 -> 90 + 5 = 95 (peak)
            (k2, t(12, 1), 90),
            (k2, t(12, 59), 5),
        ];

        let peak = compute_peak_tpm(&rows);
        assert_eq!(peak.get(&k1).copied(), Some(300));
        assert_eq!(peak.get(&k2).copied(), Some(95));
    }

    #[test]
    fn peak_tpm_handles_null_key_and_empty() {
        assert!(compute_peak_tpm(&[]).is_empty());

        let t = Utc.with_ymd_and_hms(2026, 5, 19, 0, 0, 0).unwrap();
        let rows = vec![(None, t, 7), (None, t, 3)];
        let peak = compute_peak_tpm(&rows);
        assert_eq!(peak.get(&None).copied(), Some(10));
    }

    #[test]
    fn peak_tpm_buckets_strictly_by_minute() {
        let k = Some(Uuid::from_u128(42));
        let a = Utc.with_ymd_and_hms(2026, 5, 19, 10, 0, 59).unwrap();
        let b = Utc.with_ymd_and_hms(2026, 5, 19, 10, 1, 0).unwrap();
        // Same key, adjacent seconds straddling a minute boundary -> separate buckets, not summed.
        let peak = compute_peak_tpm(&[(k, a, 5), (k, b, 6)]);
        assert_eq!(peak.get(&k).copied(), Some(6));
    }
}
