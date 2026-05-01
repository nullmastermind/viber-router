use deadpool_redis::Pool;
use deadpool_redis::redis::cmd;
use uuid::Uuid;

/// Check if the circuit is open for this group-server pair.
pub async fn is_circuit_open(redis: &Pool, group_id: Uuid, server_id: Uuid) -> bool {
    let key = format!("cb:open:{group_id}:{server_id}");
    let Ok(mut conn) = redis.get().await else {
        return false; // fail open
    };
    let result: Result<bool, _> = cmd("EXISTS").arg(&key).query_async(&mut conn).await;
    result.unwrap_or(false)
}

/// Record an error for this group-server pair.
/// Returns true if the circuit was newly tripped (first time reaching threshold).
pub async fn record_error(
    redis: &Pool,
    group_id: Uuid,
    server_id: Uuid,
    max_failures: i32,
    window_seconds: i32,
    cooldown_seconds: i32,
) -> bool {
    let err_key = format!("cb:err:{group_id}:{server_id}");
    let open_key = format!("cb:open:{group_id}:{server_id}");
    let realerted_key = format!("cb:realerted:{group_id}:{server_id}");

    let Ok(mut conn) = redis.get().await else {
        return false;
    };

    // INCR error counter
    let count: i64 = match cmd("INCR").arg(&err_key).query_async(&mut conn).await {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Set TTL if this is a new key (TTL == -1)
    if count == 1 {
        let _: Result<(), _> = cmd("EXPIRE")
            .arg(&err_key)
            .arg(window_seconds)
            .query_async(&mut conn)
            .await;
    }

    // Check threshold
    if count >= max_failures as i64 {
        // Trip the circuit — use NX to avoid resetting TTL if already open
        let newly_set: Option<String> = cmd("SET")
            .arg(&open_key)
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(cooldown_seconds)
            .query_async(&mut conn)
            .await
            .unwrap_or(None);

        // Delete error counter
        let _: Result<(), _> = cmd("DEL").arg(&err_key).query_async(&mut conn).await;

        // Set the realerted marker when circuit opens, so check_re_enabled
        // knows the circuit was previously open (prevents phantom alerts)
        if newly_set.is_some() {
            let _: Result<(), _> = cmd("SET")
                .arg(&realerted_key)
                .arg("1")
                .arg("EX")
                .arg(cooldown_seconds + 120) // outlive the open key
                .query_async(&mut conn)
                .await;
        }

        // Only send alert if we were the ones to trip it
        return newly_set.is_some();
    }

    false
}

/// Check if a circuit was re-enabled (open key expired after being tripped).
/// Returns true if re-enabled and this is the first detection.
pub async fn check_re_enabled(redis: &Pool, group_id: Uuid, server_id: Uuid) -> bool {
    let open_key = format!("cb:open:{group_id}:{server_id}");
    let realerted_key = format!("cb:realerted:{group_id}:{server_id}");

    let Ok(mut conn) = redis.get().await else {
        return false;
    };

    // If circuit is still open, not re-enabled
    let is_open: bool = cmd("EXISTS")
        .arg(&open_key)
        .query_async(&mut conn)
        .await
        .unwrap_or(true);
    if is_open {
        return false;
    }

    // Circuit is closed — check if realerted marker exists (meaning it was previously open)
    // Use DEL to atomically check-and-remove: if key existed we get 1, otherwise 0
    let deleted: i64 = cmd("DEL")
        .arg(&realerted_key)
        .query_async(&mut conn)
        .await
        .unwrap_or(0);

    // If marker existed and was deleted, the circuit was previously open and is now re-enabled
    deleted > 0
}
