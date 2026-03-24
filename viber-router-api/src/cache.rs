use deadpool_redis::Pool;
use deadpool_redis::redis::AsyncCommands;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::GroupConfig;

const KEY_PREFIX: &str = "group:config:";

pub async fn get_group_config(redis: &Pool, api_key: &str) -> Option<GroupConfig> {
    let mut conn = redis.get().await.ok()?;
    let data: Option<String> = conn.get(format!("{KEY_PREFIX}{api_key}")).await.ok()?;
    data.and_then(|d| serde_json::from_str(&d).ok())
}

pub async fn set_group_config(redis: &Pool, api_key: &str, config: &GroupConfig) {
    if let Ok(mut conn) = redis.get().await
        && let Ok(data) = serde_json::to_string(config)
    {
        let _: Result<(), _> = conn.set(format!("{KEY_PREFIX}{api_key}"), data).await;
    }
}

pub async fn invalidate_group_config(redis: &Pool, api_key: &str) {
    if let Ok(mut conn) = redis.get().await {
        let _: Result<(), _> = conn.del(format!("{KEY_PREFIX}{api_key}")).await;
    }
}

pub async fn invalidate_groups_by_server(redis: &Pool, db: &PgPool, server_id: Uuid) {
    let api_keys: Vec<(String,)> = sqlx::query_as(
        "SELECT g.api_key FROM groups g \
         JOIN group_servers gs ON g.id = gs.group_id \
         WHERE gs.server_id = $1 \
         UNION \
         SELECT g.api_key FROM groups g \
         WHERE g.count_tokens_server_id = $1",
    )
    .bind(server_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    for (api_key,) in api_keys {
        invalidate_group_config(redis, &api_key).await;
    }
}
