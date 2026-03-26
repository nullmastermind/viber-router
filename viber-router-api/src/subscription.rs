use chrono::Utc;
use deadpool_redis::redis::AsyncCommands;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::KeySubscription;
use crate::routes::{AppState, ModelPricing};

/// Result of the pre-request subscription check.
pub enum SubCheckResult {
    /// No subscriptions exist for this key — unlimited usage.
    Unlimited,
    /// A subscription was selected for charging.
    Allowed { subscription_id: Uuid },
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
        "SELECT * FROM key_subscriptions WHERE group_key_id = $1 AND status = 'active' ORDER BY created_at ASC",
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
    let key = if sub.sub_type == "hourly_reset" {
        if let (Some(activated_at), Some(reset_hours)) = (sub.activated_at, sub.reset_hours) {
            let window_idx = compute_window_idx(activated_at, reset_hours);
            format!("sub_cost:{}:w:{window_idx}", sub.id)
        } else {
            return 0.0; // Not yet activated
        }
    } else {
        format!("sub_cost:{}", sub.id)
    };

    if let Ok(mut conn) = state.redis.get().await {
        let val: Result<Option<f64>, _> = conn.get(&key).await;
        if let Ok(Some(cost)) = val {
            return cost;
        }
    }

    // Rebuild from DB
    let cost = rebuild_total_cost(&state.db, sub).await;

    // Cache it
    if let Ok(mut conn) = state.redis.get().await {
        let _: Result<(), _> = conn.set(&key, cost).await;
        if sub.sub_type == "hourly_reset"
            && let Some(reset_hours) = sub.reset_hours
        {
            let _: Result<(), _> = conn.expire(&key, (reset_hours as i64) * 3600).await;
        }
    }

    cost
}

/// Get per-model cost for a subscription from Redis, rebuilding from DB on cache miss.
async fn get_model_cost(state: &AppState, sub: &KeySubscription, model: &str) -> f64 {
    let key = if sub.sub_type == "hourly_reset" {
        if let (Some(activated_at), Some(reset_hours)) = (sub.activated_at, sub.reset_hours) {
            let window_idx = compute_window_idx(activated_at, reset_hours);
            format!("sub_cost:{}:w:{window_idx}:m:{model}", sub.id)
        } else {
            return 0.0;
        }
    } else {
        format!("sub_cost:{}:m:{model}", sub.id)
    };

    if let Ok(mut conn) = state.redis.get().await {
        let val: Result<Option<f64>, _> = conn.get(&key).await;
        if let Ok(Some(cost)) = val {
            return cost;
        }
    }

    // Rebuild from DB
    let cost = rebuild_model_cost(&state.db, sub, model).await;

    if let Ok(mut conn) = state.redis.get().await {
        let _: Result<(), _> = conn.set(&key, cost).await;
        if sub.sub_type == "hourly_reset"
            && let Some(reset_hours) = sub.reset_hours
        {
            let _: Result<(), _> = conn.expire(&key, (reset_hours as i64) * 3600).await;
        }
    }

    cost
}

async fn rebuild_total_cost(db: &PgPool, sub: &KeySubscription) -> f64 {
    if sub.sub_type == "hourly_reset" {
        if let (Some(activated_at), Some(reset_hours)) = (sub.activated_at, sub.reset_hours) {
            let window_idx = compute_window_idx(activated_at, reset_hours);
            let window_start = activated_at + chrono::Duration::seconds((window_idx as i64) * (reset_hours as i64) * 3600);
            let window_end = window_start + chrono::Duration::seconds((reset_hours as i64) * 3600);
            sqlx::query_scalar::<_, f64>(
                "SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage_logs \
                 WHERE subscription_id = $1 AND created_at >= $2 AND created_at < $3",
            )
            .bind(sub.id)
            .bind(window_start)
            .bind(window_end)
            .fetch_one(db)
            .await
            .unwrap_or(0.0)
        } else {
            0.0
        }
    } else {
        sqlx::query_scalar::<_, f64>(
            "SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage_logs WHERE subscription_id = $1",
        )
        .bind(sub.id)
        .fetch_one(db)
        .await
        .unwrap_or(0.0)
    }
}

async fn rebuild_model_cost(db: &PgPool, sub: &KeySubscription, model: &str) -> f64 {
    if sub.sub_type == "hourly_reset" {
        if let (Some(activated_at), Some(reset_hours)) = (sub.activated_at, sub.reset_hours) {
            let window_idx = compute_window_idx(activated_at, reset_hours);
            let window_start = activated_at + chrono::Duration::seconds((window_idx as i64) * (reset_hours as i64) * 3600);
            let window_end = window_start + chrono::Duration::seconds((reset_hours as i64) * 3600);
            sqlx::query_scalar::<_, f64>(
                "SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage_logs \
                 WHERE subscription_id = $1 AND model = $2 AND created_at >= $3 AND created_at < $4",
            )
            .bind(sub.id)
            .bind(model)
            .bind(window_start)
            .bind(window_end)
            .fetch_one(db)
            .await
            .unwrap_or(0.0)
        } else {
            0.0
        }
    } else {
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
}

fn compute_window_idx(activated_at: chrono::DateTime<Utc>, reset_hours: i32) -> u64 {
    let elapsed = (Utc::now() - activated_at).num_seconds().max(0) as u64;
    elapsed / ((reset_hours as u64) * 3600)
}

/// Pre-request subscription check. Returns which subscription to charge (if any).
pub async fn check_subscriptions(
    state: &AppState,
    group_key_id: Uuid,
    model: Option<&str>,
) -> SubCheckResult {
    let subs = load_subscriptions(state, group_key_id).await;
    if subs.is_empty() {
        return SubCheckResult::Unlimited;
    }

    // Sort: hourly_reset first, then fixed, FIFO within same type
    let mut sorted = subs;
    sorted.sort_by(|a, b| {
        let type_order = |t: &str| -> u8 {
            if t == "hourly_reset" { 0 } else { 1 }
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
            if sub.sub_type == "hourly_reset" {
                // Window full — skip but don't mark exhausted
                continue;
            }
            // Fixed subscription exhausted — mark permanently
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

        // This subscription has budget — select it
        return SubCheckResult::Allowed { subscription_id: sub.id };
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
) -> f64 {
    let inp = (input_tokens as f64) * pricing.input_1m_usd * rate_input;
    let out = (output_tokens as f64) * pricing.output_1m_usd * rate_output;
    let cw = (cache_creation_tokens.unwrap_or(0) as f64) * pricing.cache_write_1m_usd * rate_cache_write;
    let cr = (cache_read_tokens.unwrap_or(0) as f64) * pricing.cache_read_1m_usd * rate_cache_read;
    (inp + out + cw + cr) / 1_000_000.0
}

/// Update Redis cost counters after a successful request.
pub async fn update_cost_counters(
    state: &AppState,
    sub_id: Uuid,
    model: &str,
    cost: f64,
    sub_type: &str,
    activated_at: Option<chrono::DateTime<Utc>>,
    reset_hours: Option<i32>,
) {
    let Ok(mut conn) = state.redis.get().await else {
        tracing::warn!("Redis unavailable for cost counter update, sub_id={sub_id}, cost={cost}");
        return;
    };

    if sub_type == "hourly_reset" {
        if let (Some(act), Some(rh)) = (activated_at, reset_hours) {
            let window_idx = compute_window_idx(act, rh);
            let ttl = (rh as i64) * 3600;

            // Window total
            let wkey = format!("sub_cost:{sub_id}:w:{window_idx}");
            let _: Result<f64, _> = deadpool_redis::redis::cmd("INCRBYFLOAT")
                .arg(&wkey)
                .arg(cost)
                .query_async(&mut *conn)
                .await;
            let _: Result<(), _> = conn.expire(&wkey, ttl).await;

            // Window per-model
            let wmkey = format!("sub_cost:{sub_id}:w:{window_idx}:m:{model}");
            let _: Result<f64, _> = deadpool_redis::redis::cmd("INCRBYFLOAT")
                .arg(&wmkey)
                .arg(cost)
                .query_async(&mut *conn)
                .await;
            let _: Result<(), _> = conn.expire(&wmkey, ttl).await;
        }
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
