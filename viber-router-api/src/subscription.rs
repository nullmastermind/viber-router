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

/// A bonus server extracted from a bonus subscription.
pub struct BonusServer {
    pub subscription_id: Uuid,
    pub base_url: String,
    pub api_key: String,
    pub name: String,
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
    /// One or more bonus subscriptions are active. Try bonus servers first, then fall back.
    BonusServers {
        servers: Vec<BonusServer>,
        /// The non-bonus subscription to charge if all bonus servers fail (sub_id, rpm_limit).
        fallback_subscription: Option<(Uuid, Option<f64>)>,
    },
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

    // Separate bonus subs from non-bonus subs
    let (bonus_subs, non_bonus_subs): (Vec<_>, Vec<_>) =
        active.into_iter().partition(|s| s.sub_type == "bonus");

    // Run existing non-bonus logic on non-bonus subs to derive fallback_subscription
    // Sort: hourly_reset first, then pay_per_request, then fixed, FIFO within same type
    let mut sorted = non_bonus_subs;
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

    let mut fallback_subscription: Option<(Uuid, Option<f64>)> = None;
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

        // This non-bonus subscription has budget — it's the fallback
        fallback_subscription = Some((sub.id, sub.rpm_limit));
        break;
    }

    // Collect active bonus subs sorted by created_at ASC
    let bonus_servers: Vec<BonusServer> = bonus_subs
        .into_iter()
        .filter_map(|sub| {
            let base_url = sub.bonus_base_url?;
            let api_key = sub.bonus_api_key?;
            let name = sub.bonus_name.unwrap_or_default();
            Some(BonusServer {
                subscription_id: sub.id,
                base_url,
                api_key,
                name,
            })
        })
        .collect();
    // bonus_servers already ordered by created_at ASC from load_subscriptions (FIFO)

    // If non-empty bonus subs — return BonusServers
    if !bonus_servers.is_empty() {
        return SubCheckResult::BonusServers {
            servers: bonus_servers,
            fallback_subscription,
        };
    }

    // No bonus subs — return existing result based on fallback_subscription
    if let Some((sub_id, rpm_limit)) = fallback_subscription {
        SubCheckResult::Allowed { subscription_id: sub_id, rpm_limit }
    } else {
        SubCheckResult::Blocked
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::ModelPricing;

    #[test]
    fn test_compute_rpm_window_above_one() {
        assert_eq!(compute_rpm_window(5.0), (60, 5));
        assert_eq!(compute_rpm_window(1.0), (60, 1));
        assert_eq!(compute_rpm_window(10.5), (60, 10));
    }

    #[test]
    fn test_compute_rpm_window_below_one() {
        assert_eq!(compute_rpm_window(0.5), (120, 1));
        assert_eq!(compute_rpm_window(0.1), (600, 1));
    }

    #[test]
    fn test_calculate_cost_basic() {
        let pricing = ModelPricing {
            input_1m_usd: 3.0,
            output_1m_usd: 15.0,
            cache_write_1m_usd: 3.75,
            cache_read_1m_usd: 0.30,
        };
        let cost = calculate_cost(
            &pricing,
            1.0, 1.0, 1.0, 1.0, // rates = 1x
            1000, 500, None, None, false,
        );
        // input: 1000 * 3.0 / 1M = 0.003
        // output: 500 * 15.0 / 1M = 0.0075
        let expected = (1000.0 * 3.0 + 500.0 * 15.0) / 1_000_000.0;
        assert!((cost - expected).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_cost_with_cache_tokens() {
        let pricing = ModelPricing {
            input_1m_usd: 3.0,
            output_1m_usd: 15.0,
            cache_write_1m_usd: 3.75,
            cache_read_1m_usd: 0.30,
        };
        let cost = calculate_cost(
            &pricing,
            1.0, 1.0, 1.0, 1.0,
            1000, 500, Some(200), Some(300), false,
        );
        let expected = (1000.0 * 3.0 + 500.0 * 15.0 + 200.0 * 3.75 + 300.0 * 0.30) / 1_000_000.0;
        assert!((cost - expected).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_cost_with_rates() {
        let pricing = ModelPricing {
            input_1m_usd: 3.0,
            output_1m_usd: 15.0,
            cache_write_1m_usd: 3.75,
            cache_read_1m_usd: 0.30,
        };
        let cost = calculate_cost(
            &pricing,
            0.5, 2.0, 1.0, 1.0, // input at 50%, output at 200%
            1000, 500, None, None, false,
        );
        let expected = (1000.0 * 3.0 * 0.5 + 500.0 * 15.0 * 2.0) / 1_000_000.0;
        assert!((cost - expected).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_cost_normalize_cache_read() {
        let pricing = ModelPricing {
            input_1m_usd: 3.0,
            output_1m_usd: 15.0,
            cache_write_1m_usd: 3.75,
            cache_read_1m_usd: 0.30,
        };
        // With normalize_cache_read=true, cache_read uses input price instead
        let cost = calculate_cost(
            &pricing,
            1.0, 1.0, 1.0, 1.0,
            1000, 500, None, Some(300), true,
        );
        // cache_read: 300 * input_1m_usd(3.0) * rate_input(1.0) instead of cache_read_1m_usd
        let expected = (1000.0 * 3.0 + 500.0 * 15.0 + 300.0 * 3.0) / 1_000_000.0;
        assert!((cost - expected).abs() < 1e-10);
    }

    #[test]
    fn test_bonus_server_from_key_subscription() {
        let sub = KeySubscription {
            id: Uuid::new_v4(),
            group_key_id: Uuid::new_v4(),
            plan_id: None,
            sub_type: "bonus".to_string(),
            cost_limit_usd: 0.0,
            model_limits: serde_json::json!({}),
            model_request_costs: serde_json::json!({}),
            reset_hours: None,
            duration_days: 36500,
            rpm_limit: None,
            status: "active".to_string(),
            activated_at: None,
            expires_at: None,
            created_at: chrono::Utc::now(),
            bonus_base_url: Some("https://api.anthropic.com".to_string()),
            bonus_api_key: Some("sk-ant-test123".to_string()),
            bonus_name: Some("Claude Code Max".to_string()),
            bonus_quota_url: None,
            bonus_quota_headers: None,
        };

        // Simulate the filter_map logic from check_subscriptions
        let server = BonusServer {
            subscription_id: sub.id,
            base_url: sub.bonus_base_url.clone().unwrap(),
            api_key: sub.bonus_api_key.clone().unwrap(),
            name: sub.bonus_name.clone().unwrap_or_default(),
        };

        assert_eq!(server.base_url, "https://api.anthropic.com");
        assert_eq!(server.api_key, "sk-ant-test123");
        assert_eq!(server.name, "Claude Code Max");
        assert_eq!(server.subscription_id, sub.id);
    }

    #[test]
    fn test_bonus_server_skipped_when_missing_fields() {
        let sub = KeySubscription {
            id: Uuid::new_v4(),
            group_key_id: Uuid::new_v4(),
            plan_id: None,
            sub_type: "bonus".to_string(),
            cost_limit_usd: 0.0,
            model_limits: serde_json::json!({}),
            model_request_costs: serde_json::json!({}),
            reset_hours: None,
            duration_days: 36500,
            rpm_limit: None,
            status: "active".to_string(),
            activated_at: None,
            expires_at: None,
            created_at: chrono::Utc::now(),
            bonus_base_url: None,  // Missing!
            bonus_api_key: Some("sk-ant-test123".to_string()),
            bonus_name: Some("Broken Bonus".to_string()),
            bonus_quota_url: None,
            bonus_quota_headers: None,
        };

        // Simulate the filter_map — should return None due to missing base_url
        let result = sub.bonus_base_url.as_ref().and_then(|base_url| {
            sub.bonus_api_key.as_ref().map(|api_key| BonusServer {
                subscription_id: sub.id,
                base_url: base_url.clone(),
                api_key: api_key.clone(),
                name: sub.bonus_name.clone().unwrap_or_default(),
            })
        });

        assert!(result.is_none());
    }

    #[test]
    fn test_bonus_default_name_when_missing() {
        let sub = KeySubscription {
            id: Uuid::new_v4(),
            group_key_id: Uuid::new_v4(),
            plan_id: None,
            sub_type: "bonus".to_string(),
            cost_limit_usd: 0.0,
            model_limits: serde_json::json!({}),
            model_request_costs: serde_json::json!({}),
            reset_hours: None,
            duration_days: 36500,
            rpm_limit: None,
            status: "active".to_string(),
            activated_at: None,
            expires_at: None,
            created_at: chrono::Utc::now(),
            bonus_base_url: Some("https://api.example.com".to_string()),
            bonus_api_key: Some("key-123".to_string()),
            bonus_name: None, // Name is None
            bonus_quota_url: None,
            bonus_quota_headers: None,
        };

        let name = sub.bonus_name.unwrap_or_default();
        assert_eq!(name, "");
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
