use chrono::{TimeZone, Utc};
use deadpool_redis::redis::{AsyncCommands, cmd};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::KeySubscription;
use crate::routes::{AppState, ModelPricing};

/// Get the window start epoch for a subscription from Redis.
/// Returns None if no active window exists (key missing or Redis error).
async fn get_window_start(state: &AppState, sub_id: Uuid) -> Option<i64> {
    let key = format!("sub_window_start:{sub_id}");
    let mut conn = state.redis.get().await.ok()?;
    let val: Option<String> = conn.get(&key).await.ok()?;
    val?.parse::<i64>().ok()
}

/// Ensure a window start exists for a subscription, creating one if absent (SETNX semantics).
/// Returns the stored epoch, or None on Redis error.
async fn ensure_window_start(state: &AppState, sub_id: Uuid, reset_hours: i32) -> Option<i64> {
    let key = format!("sub_window_start:{sub_id}");
    let ttl = (reset_hours as i64) * 3600;
    let now_epoch = Utc::now().timestamp();

    let mut conn = match state.redis.get().await {
        Ok(c) => c,
        Err(_) => {
            tracing::warn!("Redis unavailable in ensure_window_start, sub_id={sub_id}");
            return None;
        }
    };

    // SET NX EX — only sets if key does not exist
    let _: Result<(), _> = deadpool_redis::redis::cmd("SET")
        .arg(&key)
        .arg(now_epoch)
        .arg("NX")
        .arg("EX")
        .arg(ttl)
        .query_async(&mut *conn)
        .await;

    // GET the stored value (either the one we just set, or the pre-existing one)
    let val: Option<String> = match conn.get(&key).await {
        Ok(v) => v,
        Err(_) => {
            tracing::warn!("Redis GET failed in ensure_window_start, sub_id={sub_id}");
            return None;
        }
    };
    val?.parse::<i64>().ok()
}

/// Result of the pre-request subscription check.
pub enum SubCheckResult {
    /// A subscription was selected for charging.
    Allowed {
        subscription_id: Uuid,
        rpm_limit: Option<f64>,
    },
    /// All subscriptions are blocked (exhausted/expired/cancelled or per-model limit hit).
    Blocked,
}

/// Load active subscriptions for a key, using Redis cache with DB fallback.
async fn load_subscriptions(state: &AppState, group_key_id: Uuid) -> Vec<KeySubscription> {
    let cache_key = format!("key_subs:{group_key_id}");

    // Try Redis cache
    if let Ok(mut conn) = state.redis.get().await {
        let cached: Result<Option<String>, _> = conn.get(&cache_key).await;
        if let Ok(Some(data)) = cached
            && let Ok(subs) = serde_json::from_str::<Vec<KeySubscription>>(&data)
        {
            return subs;
        }
    }

    // DB fallback
    let subs = sqlx::query_as::<_, KeySubscription>(
        "SELECT * FROM key_subscriptions WHERE group_key_id = $1 ORDER BY created_at ASC",
    )
    .bind(group_key_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Cache the result
    if let Ok(mut conn) = state.redis.get().await
        && let Ok(data) = serde_json::to_string(&subs)
    {
        let _: Result<(), _> = conn.set_ex(&cache_key, data, 300).await;
    }

    subs
}

/// Get the current cost for a subscription from Redis, rebuilding from DB on cache miss.
pub async fn get_total_cost(state: &AppState, sub: &KeySubscription) -> f64 {
    if sub.reset_hours.is_some() {
        let ws = match get_window_start(state, sub.id).await {
            Some(ws) => ws,
            None => return 0.0, // No active window
        };
        let key = format!("sub_cost:{}:ws:{ws}", sub.id);
        if let Ok(mut conn) = state.redis.get().await {
            let val: Result<Option<f64>, _> = conn.get(&key).await;
            if let Ok(Some(cost)) = val {
                return cost;
            }
        }
        // Rebuild from DB
        rebuild_total_cost(state, sub, ws).await
    } else {
        let key = format!("sub_cost:{}", sub.id);
        if let Ok(mut conn) = state.redis.get().await {
            let val: Result<Option<f64>, _> = conn.get(&key).await;
            if let Ok(Some(cost)) = val {
                return cost;
            }
        }
        // Rebuild from DB
        let cost = rebuild_total_cost_fixed(&state.db, sub).await;
        if let Ok(mut conn) = state.redis.get().await {
            let _: Result<(), _> = conn.set(&key, cost).await;
        }
        cost
    }
}

/// Get per-model cost for a subscription from Redis, rebuilding from DB on cache miss.
async fn get_model_cost(state: &AppState, sub: &KeySubscription, model: &str) -> f64 {
    if sub.reset_hours.is_some() {
        let ws = match get_window_start(state, sub.id).await {
            Some(ws) => ws,
            None => return 0.0, // No active window
        };
        let key = format!("sub_cost:{}:ws:{ws}:m:{model}", sub.id);
        if let Ok(mut conn) = state.redis.get().await {
            let val: Result<Option<f64>, _> = conn.get(&key).await;
            if let Ok(Some(cost)) = val {
                return cost;
            }
        }
        // Rebuild from DB
        rebuild_model_cost(state, sub, model, ws).await
    } else {
        let key = format!("sub_cost:{}:m:{model}", sub.id);
        if let Ok(mut conn) = state.redis.get().await {
            let val: Result<Option<f64>, _> = conn.get(&key).await;
            if let Ok(Some(cost)) = val {
                return cost;
            }
        }
        // Rebuild from DB
        let cost = rebuild_model_cost_fixed(&state.db, sub, model).await;
        if let Ok(mut conn) = state.redis.get().await {
            let _: Result<(), _> = conn.set(&key, cost).await;
        }
        cost
    }
}

/// Rebuild total cost for a windowed subscription from DB, cache the result, and return it.
async fn rebuild_total_cost(state: &AppState, sub: &KeySubscription, window_start: i64) -> f64 {
    let reset_hours = match sub.reset_hours {
        Some(rh) => rh,
        None => return 0.0,
    };
    let ws_dt = match Utc.timestamp_opt(window_start, 0).single() {
        Some(dt) => dt,
        None => return 0.0,
    };
    let we_dt = ws_dt + chrono::Duration::seconds((reset_hours as i64) * 3600);

    let cost = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage_logs \
         WHERE subscription_id = $1 AND created_at >= $2 AND created_at < $3",
    )
    .bind(sub.id)
    .bind(ws_dt)
    .bind(we_dt)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0.0);

    let key = format!("sub_cost:{}:ws:{window_start}", sub.id);
    if let Ok(mut conn) = state.redis.get().await {
        let _: Result<(), _> = conn.set(&key, cost).await;
    }
    cost
}

/// Rebuild total cost for a fixed (non-windowed) subscription from DB.
async fn rebuild_total_cost_fixed(db: &PgPool, sub: &KeySubscription) -> f64 {
    sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage_logs WHERE subscription_id = $1",
    )
    .bind(sub.id)
    .fetch_one(db)
    .await
    .unwrap_or(0.0)
}

/// Rebuild per-model cost for a windowed subscription from DB, cache the result, and return it.
async fn rebuild_model_cost(
    state: &AppState,
    sub: &KeySubscription,
    model: &str,
    window_start: i64,
) -> f64 {
    let reset_hours = match sub.reset_hours {
        Some(rh) => rh,
        None => return 0.0,
    };
    let ws_dt = match Utc.timestamp_opt(window_start, 0).single() {
        Some(dt) => dt,
        None => return 0.0,
    };
    let we_dt = ws_dt + chrono::Duration::seconds((reset_hours as i64) * 3600);

    let cost = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage_logs \
         WHERE subscription_id = $1 AND model = $2 AND created_at >= $3 AND created_at < $4",
    )
    .bind(sub.id)
    .bind(model)
    .bind(ws_dt)
    .bind(we_dt)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0.0);

    let key = format!("sub_cost:{}:ws:{window_start}:m:{model}", sub.id);
    if let Ok(mut conn) = state.redis.get().await {
        let _: Result<(), _> = conn.set(&key, cost).await;
    }
    cost
}

/// Rebuild per-model cost for a fixed (non-windowed) subscription from DB.
async fn rebuild_model_cost_fixed(db: &PgPool, sub: &KeySubscription, model: &str) -> f64 {
    sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage_logs \
         WHERE subscription_id = $1 AND model = $2",
    )
    .bind(sub.id)
    .bind(model)
    .fetch_one(db)
    .await
    .unwrap_or(0.0)
}

/// Pre-request subscription check. Returns which subscription to charge (if any).
pub async fn check_subscriptions(
    state: &AppState,
    group_key_id: Uuid,
    model: Option<&str>,
) -> SubCheckResult {
    let all_subs = load_subscriptions(state, group_key_id).await;
    let active: Vec<_> = all_subs.into_iter().filter(|s| s.status == "active").collect();
    if active.is_empty() {
        return SubCheckResult::Blocked;
    }

    // Sort: hourly_reset first, then pay_per_request, then fixed, FIFO within same type
    let mut sorted = active;
    sorted.sort_by(|a, b| {
        let type_order = |t: &str| -> u8 {
            match t {
                "hourly_reset" => 0,
                "pay_per_request" => 1,
                _ => 2,
            }
        };
        type_order(&a.sub_type)
            .cmp(&type_order(&b.sub_type))
            .then(a.created_at.cmp(&b.created_at))
    });

    let now = Utc::now();

    for sub in &sorted {
        // Check expiration
        if let Some(expires_at) = sub.expires_at
            && now > expires_at
        {
            // Mark expired in DB (fire-and-forget)
            let db = state.db.clone();
            let sub_id = sub.id;
            let key_id = sub.group_key_id;
            let redis = state.redis.clone();
            tokio::spawn(async move {
                let _ = sqlx::query("UPDATE key_subscriptions SET status = 'expired' WHERE id = $1 AND status = 'active'")
                    .bind(sub_id)
                    .execute(&db)
                    .await;
                if let Ok(mut conn) = redis.get().await {
                    let _: Result<(), _> = conn.del(format!("key_subs:{key_id}")).await;
                }
            });
            continue;
        }

        // Check total budget
        let total_cost = get_total_cost(state, sub).await;
        if total_cost >= sub.cost_limit_usd {
            if sub.reset_hours.is_some() {
                // Window full — skip but don't mark exhausted
                continue;
            }
            // Fixed/pay_per_request subscription exhausted — mark permanently
            let db = state.db.clone();
            let sub_id = sub.id;
            let key_id = sub.group_key_id;
            let redis = state.redis.clone();
            tokio::spawn(async move {
                let _ = sqlx::query("UPDATE key_subscriptions SET status = 'exhausted' WHERE id = $1 AND status = 'active'")
                    .bind(sub_id)
                    .execute(&db)
                    .await;
                if let Ok(mut conn) = redis.get().await {
                    let _: Result<(), _> = conn.del(format!("key_subs:{key_id}")).await;
                }
            });
            continue;
        }

        // For pay_per_request: skip if model is not in model_request_costs
        if sub.sub_type == "pay_per_request"
            && let Some(model_name) = model
            && sub.model_request_costs.as_object()
                .map(|m| !m.contains_key(model_name))
                .unwrap_or(true)
        {
            continue;
        }

        // Check per-model limit
        if let Some(model_name) = model
            && let Some(limits) = sub.model_limits.as_object()
            && let Some(limit_val) = limits.get(model_name).and_then(|v| v.as_f64())
        {
            let model_cost = get_model_cost(state, sub, model_name).await;
            if model_cost >= limit_val {
                continue;
            }
        }

        // Check RPM limit
        if let Some(rpm) = sub.rpm_limit
            && rpm > 0.0
            && is_rpm_limited(state, sub.id, rpm).await
        {
            continue;
        }

        // This subscription has budget — select it
        return SubCheckResult::Allowed {
            subscription_id: sub.id,
            rpm_limit: sub.rpm_limit,
        };
    }

    SubCheckResult::Blocked
}

/// Handle lazy activation for a subscription.
pub async fn ensure_activated(state: &AppState, sub_id: Uuid, duration_days: i32) {
    let activation_key = format!("sub_activated:{sub_id}");
    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let ttl_secs = (duration_days as i64) * 86400 + 3600; // duration + 1h buffer

    if let Ok(mut conn) = state.redis.get().await {
        // SETNX with TTL — only first request sets the activation
        let set: Result<bool, _> = deadpool_redis::redis::cmd("SET")
            .arg(&activation_key)
            .arg(&now_str)
            .arg("NX")
            .arg("EX")
            .arg(ttl_secs)
            .query_async(&mut *conn)
            .await;

        if let Ok(true) = set {
            // We won the race — update DB
            let expires_at = now + chrono::Duration::days(duration_days as i64);
            let _ = sqlx::query(
                "UPDATE key_subscriptions SET activated_at = $1, expires_at = $2 WHERE id = $3 AND activated_at IS NULL",
            )
            .bind(now)
            .bind(expires_at)
            .bind(sub_id)
            .execute(&state.db)
            .await;

            // Invalidate subscription list cache
            if let Ok(gk_id) = sqlx::query_scalar::<_, Uuid>(
                "SELECT group_key_id FROM key_subscriptions WHERE id = $1",
            )
            .bind(sub_id)
            .fetch_one(&state.db)
            .await
            {
                let _: Result<(), _> = conn.del(format!("key_subs:{gk_id}")).await;
            }
        }
        // SETNX loser or already activated — no action needed.
        // The caller re-fetches the subscription from DB to get the stored activated_at.
    } else {
        // Redis unavailable — fall back to DB-only activation
        let expires_at = now + chrono::Duration::days(duration_days as i64);
        let _ = sqlx::query(
            "UPDATE key_subscriptions SET activated_at = $1, expires_at = $2 WHERE id = $3 AND activated_at IS NULL",
        )
        .bind(now)
        .bind(expires_at)
        .bind(sub_id)
        .execute(&state.db)
        .await;
        tracing::warn!("Redis unavailable for subscription activation, used DB-only fallback");
    }
}

/// Calculate cost for a request using model pricing and server rates.
#[allow(clippy::too_many_arguments)]
pub fn calculate_cost(
    pricing: &ModelPricing,
    rate_input: f64,
    rate_output: f64,
    rate_cache_write: f64,
    rate_cache_read: f64,
    input_tokens: i32,
    output_tokens: i32,
    cache_creation_tokens: Option<i32>,
    cache_read_tokens: Option<i32>,
    normalize_cache_read: bool,
) -> f64 {
    let inp = (input_tokens as f64) * pricing.input_1m_usd * rate_input;
    let out = (output_tokens as f64) * pricing.output_1m_usd * rate_output;
    let cw = (cache_creation_tokens.unwrap_or(0) as f64) * pricing.cache_write_1m_usd * rate_cache_write;
    let cr = if normalize_cache_read {
        (cache_read_tokens.unwrap_or(0) as f64) * pricing.input_1m_usd * rate_input
    } else {
        (cache_read_tokens.unwrap_or(0) as f64) * pricing.cache_read_1m_usd * rate_cache_read
    };
    (inp + out + cw + cr) / 1_000_000.0
}

/// Update Redis cost counters after a successful request.
pub async fn update_cost_counters(
    state: &AppState,
    sub_id: Uuid,
    model: &str,
    cost: f64,
    reset_hours: Option<i32>,
) {
    let Ok(mut conn) = state.redis.get().await else {
        tracing::warn!("Redis unavailable for cost counter update, sub_id={sub_id}, cost={cost}");
        return;
    };

    if let Some(rh) = reset_hours {
        // Demand-driven windowing: ensure a window start exists
        let ws = match ensure_window_start(state, sub_id, rh).await {
            Some(ws) => ws,
            None => {
                tracing::warn!(
                    "Could not ensure window start for sub_id={sub_id}, skipping cost counter update"
                );
                return;
            }
        };
        let ttl = (rh as i64) * 3600;

        // Window total
        let wkey = format!("sub_cost:{sub_id}:ws:{ws}");
        let _: Result<f64, _> = deadpool_redis::redis::cmd("INCRBYFLOAT")
            .arg(&wkey)
            .arg(cost)
            .query_async(&mut *conn)
            .await;
        let _: Result<(), _> = conn.expire(&wkey, ttl).await;

        // Window per-model
        let wmkey = format!("sub_cost:{sub_id}:ws:{ws}:m:{model}");
        let _: Result<f64, _> = deadpool_redis::redis::cmd("INCRBYFLOAT")
            .arg(&wmkey)
            .arg(cost)
            .query_async(&mut *conn)
            .await;
        let _: Result<(), _> = conn.expire(&wmkey, ttl).await;
    } else {
        // Fixed total
        let tkey = format!("sub_cost:{sub_id}");
        let _: Result<f64, _> = deadpool_redis::redis::cmd("INCRBYFLOAT")
            .arg(&tkey)
            .arg(cost)
            .query_async(&mut *conn)
            .await;

        // Fixed per-model
        let mkey = format!("sub_cost:{sub_id}:m:{model}");
        let _: Result<f64, _> = deadpool_redis::redis::cmd("INCRBYFLOAT")
            .arg(&mkey)
            .arg(cost)
            .query_async(&mut *conn)
            .await;
    }
}

/// Compute the RPM window parameters from a float RPM value.
/// Returns (window_seconds, max_requests).
/// RPM < 1: scale window up, max = 1. RPM >= 1: window = 60s, max = floor(rpm).
pub fn compute_rpm_window(rpm: f64) -> (i64, i64) {
    if rpm < 1.0 {
        let window = (60.0 / rpm).ceil() as i64;
        (window, 1)
    } else {
        (60, rpm.floor() as i64)
    }
}

/// Check if a subscription has exceeded its RPM limit.
/// Fails open on Redis errors (returns false).
async fn is_rpm_limited(state: &AppState, sub_id: Uuid, rpm: f64) -> bool {
    let key = format!("sub_rpm:{sub_id}");
    let Ok(mut conn) = state.redis.get().await else {
        return false; // fail open
    };
    let count: Option<i64> = match cmd("GET").arg(&key).query_async(&mut *conn).await {
        Ok(c) => c,
        Err(_) => return false, // fail open
    };
    let (_, max_requests) = compute_rpm_window(rpm);
    count.unwrap_or(0) >= max_requests
}

/// Increment the RPM counter for a subscription after selecting it for a request.
/// Always sets TTL to ensure the key expires even if a previous EXPIRE failed.
/// Silently skips on Redis errors.
pub async fn increment_rpm(state: &AppState, sub_id: Uuid, rpm: f64) {
    let key = format!("sub_rpm:{sub_id}");
    let Ok(mut conn) = state.redis.get().await else {
        return;
    };
    let _: Result<i64, _> = cmd("INCR").arg(&key).query_async(&mut *conn).await;
    let (window_seconds, _) = compute_rpm_window(rpm);
    let _: Result<(), _> = cmd("EXPIRE")
        .arg(&key)
        .arg(window_seconds)
        .query_async(&mut *conn)
        .await;
}
