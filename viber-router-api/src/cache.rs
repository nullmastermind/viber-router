use deadpool_redis::Pool;
use deadpool_redis::redis::AsyncCommands;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::GroupConfig;

const KEY_PREFIX: &str = "group:config:";
const CONFIG_TTL_SECS: i64 = 60;

pub async fn get_group_config(redis: &Pool, api_key: &str) -> Option<GroupConfig> {
    let mut conn = redis.get().await.ok()?;
    let data: Option<String> = conn.get(format!("{KEY_PREFIX}{api_key}")).await.ok()?;
    data.and_then(|d| serde_json::from_str(&d).ok())
}

pub async fn set_group_config(redis: &Pool, api_key: &str, config: &GroupConfig) {
    if let Ok(mut conn) = redis.get().await
        && let Ok(data) = serde_json::to_string(config)
    {
        let _: Result<(), _> = conn.set_ex(format!("{KEY_PREFIX}{api_key}"), data, CONFIG_TTL_SECS as u64).await;
    }
}

pub async fn invalidate_group_config(redis: &Pool, api_key: &str) {
    if let Ok(mut conn) = redis.get().await {
        let _: Result<(), _> = conn.del(format!("{KEY_PREFIX}{api_key}")).await;
    }
}

/// Invalidate cache for a group's master key AND all its sub-keys.
pub async fn invalidate_group_all_keys(redis: &Pool, db: &PgPool, group_id: Uuid) {
    // Get master key
    let master: Option<(String,)> = sqlx::query_as(
        "SELECT api_key FROM groups WHERE id = $1",
    )
    .bind(group_id)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    if let Some((master_key,)) = master {
        invalidate_group_config(redis, &master_key).await;
    }

    // Get all sub-keys
    let sub_keys: Vec<(String,)> = sqlx::query_as(
        "SELECT api_key FROM group_keys WHERE group_id = $1",
    )
    .bind(group_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    for (api_key,) in sub_keys {
        invalidate_group_config(redis, &api_key).await;
    }
}

pub async fn invalidate_groups_by_server(redis: &Pool, db: &PgPool, server_id: Uuid) {
    // Get all group IDs affected by this server
    let group_ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT g.id FROM groups g \
         JOIN group_servers gs ON g.id = gs.group_id \
         WHERE gs.server_id = $1 \
         UNION \
         SELECT g.id FROM groups g \
         WHERE g.count_tokens_server_id = $1",
    )
    .bind(server_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    for (gid,) in group_ids {
        invalidate_group_all_keys(redis, db, gid).await;
    }
}
