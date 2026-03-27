use deadpool_redis::Pool;
use deadpool_redis::redis::cmd;
use uuid::Uuid;

/// Check if the rate limit has been reached for this group-server pair.
/// Returns true if the server should be skipped (limit reached).
/// Fails open on Redis errors (returns false).
pub async fn is_rate_limited(
    redis: &Pool,
    group_id: Uuid,
    server_id: Uuid,
    max_requests: i32,
) -> bool {
    let key = format!("rl:{group_id}:{server_id}");
    let Ok(mut conn) = redis.get().await else {
        return false; // fail open
    };
    let count: Option<i64> = match cmd("GET").arg(&key).query_async(&mut conn).await {
        Ok(c) => c,
        Err(_) => return false, // fail open on actual Redis error
    };
    count.unwrap_or(0) >= max_requests as i64
}

/// Increment the rate limit counter for this group-server pair.
/// Sets TTL on the key if it's newly created (count == 1 after INCR).
/// Silently skips on Redis errors.
pub async fn increment_rate_limit(
    redis: &Pool,
    group_id: Uuid,
    server_id: Uuid,
    window_seconds: i32,
) {
    let key = format!("rl:{group_id}:{server_id}");
    let Ok(mut conn) = redis.get().await else {
        return;
    };
    let count: i64 = match cmd("INCR").arg(&key).query_async(&mut conn).await {
        Ok(c) => c,
        Err(_) => return,
    };
    if count == 1 {
        let _: Result<(), _> = cmd("EXPIRE")
            .arg(&key)
            .arg(window_seconds)
            .query_async(&mut conn)
            .await;
    }
}

const IP_RATE_LIMIT: i64 = 30;
const IP_RATE_WINDOW: i32 = 60;

/// Check if an IP has exceeded the public endpoint rate limit (30 req/60s).
/// Fails open on Redis errors.
pub async fn is_ip_rate_limited(redis: &Pool, ip: &str) -> bool {
    let key = format!("rl:pub:{ip}");
    let Ok(mut conn) = redis.get().await else {
        return false;
    };
    let count: Option<i64> = match cmd("GET").arg(&key).query_async(&mut conn).await {
        Ok(c) => c,
        Err(_) => return false,
    };
    count.unwrap_or(0) >= IP_RATE_LIMIT
}

/// Increment the IP rate limit counter. Sets 60s TTL on new keys.
/// Silently skips on Redis errors.
pub async fn increment_ip_rate_limit(redis: &Pool, ip: &str) {
    let key = format!("rl:pub:{ip}");
    let Ok(mut conn) = redis.get().await else {
        return;
    };
    let count: i64 = match cmd("INCR").arg(&key).query_async(&mut conn).await {
        Ok(c) => c,
        Err(_) => return,
    };
    if count == 1 {
        let _: Result<(), _> = cmd("EXPIRE")
            .arg(&key)
            .arg(IP_RATE_WINDOW)
            .query_async(&mut conn)
            .await;
    }
}
