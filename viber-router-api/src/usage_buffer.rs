use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TokenUsageEntry {
    pub group_id: Uuid,
    pub server_id: Uuid,
    pub model: Option<String>,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub cache_creation_tokens: Option<i32>,
    pub cache_read_tokens: Option<i32>,
    pub is_dynamic_key: bool,
    pub key_hash: Option<String>,
    pub group_key_id: Option<Uuid>,
    pub cost_usd: Option<f64>,
    pub subscription_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub content_hash: Option<String>,
}

pub fn hash_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string()
}

const BATCH_SIZE: usize = 100;
const FLUSH_INTERVAL_SECS: u64 = 5;

pub async fn flush_task(mut rx: mpsc::Receiver<TokenUsageEntry>, pool: PgPool) {
    let mut buffer: Vec<TokenUsageEntry> = Vec::with_capacity(BATCH_SIZE);
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(FLUSH_INTERVAL_SECS));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if !buffer.is_empty() {
                    flush_batch(&pool, &buffer).await;
                    buffer.clear();
                }
            }
            entry = rx.recv() => {
                match entry {
                    Some(e) => {
                        buffer.push(e);
                        if buffer.len() >= BATCH_SIZE {
                            flush_batch(&pool, &buffer).await;
                            buffer.clear();
                        }
                    }
                    None => {
                        if !buffer.is_empty() {
                            flush_batch(&pool, &buffer).await;
                        }
                        break;
                    }
                }
            }
        }
    }
}

async fn flush_batch(pool: &PgPool, entries: &[TokenUsageEntry]) {
    let len = entries.len();
    let mut ids = Vec::with_capacity(len);
    let mut created_ats = Vec::with_capacity(len);
    let mut group_ids = Vec::with_capacity(len);
    let mut server_ids = Vec::with_capacity(len);
    let mut models: Vec<Option<String>> = Vec::with_capacity(len);
    let mut input_tokens_vec = Vec::with_capacity(len);
    let mut output_tokens_vec = Vec::with_capacity(len);
    let mut cache_creation_vec: Vec<Option<i32>> = Vec::with_capacity(len);
    let mut cache_read_vec: Vec<Option<i32>> = Vec::with_capacity(len);
    let mut is_dynamic_keys = Vec::with_capacity(len);
    let mut key_hashes: Vec<Option<String>> = Vec::with_capacity(len);
    let mut group_key_ids: Vec<Option<Uuid>> = Vec::with_capacity(len);
    let mut cost_usds: Vec<Option<f64>> = Vec::with_capacity(len);
    let mut subscription_ids: Vec<Option<Uuid>> = Vec::with_capacity(len);
    let mut content_hashes: Vec<Option<String>> = Vec::with_capacity(len);

    for e in entries {
        ids.push(Uuid::new_v4());
        created_ats.push(e.created_at);
        group_ids.push(e.group_id);
        server_ids.push(e.server_id);
        models.push(e.model.clone());
        input_tokens_vec.push(e.input_tokens);
        output_tokens_vec.push(e.output_tokens);
        cache_creation_vec.push(e.cache_creation_tokens);
        cache_read_vec.push(e.cache_read_tokens);
        is_dynamic_keys.push(e.is_dynamic_key);
        key_hashes.push(e.key_hash.clone());
        group_key_ids.push(e.group_key_id);
        cost_usds.push(e.cost_usd);
        subscription_ids.push(e.subscription_id);
        content_hashes.push(e.content_hash.clone());
    }

    let result = sqlx::query(
        "INSERT INTO token_usage_logs \
         (id, created_at, group_id, server_id, model, input_tokens, output_tokens, \
          cache_creation_tokens, cache_read_tokens, is_dynamic_key, key_hash, group_key_id, \
          cost_usd, subscription_id, content_hash) \
         SELECT * FROM UNNEST(\
           $1::uuid[], $2::timestamptz[], $3::uuid[], $4::uuid[], \
           $5::text[], $6::integer[], $7::integer[], \
           $8::integer[], $9::integer[], $10::boolean[], $11::text[], $12::uuid[], \
           $13::float8[], $14::uuid[], $15::text[])",
    )
    .bind(&ids)
    .bind(&created_ats)
    .bind(&group_ids)
    .bind(&server_ids)
    .bind(&models)
    .bind(&input_tokens_vec)
    .bind(&output_tokens_vec)
    .bind(&cache_creation_vec)
    .bind(&cache_read_vec)
    .bind(&is_dynamic_keys)
    .bind(&key_hashes)
    .bind(&group_key_ids)
    .bind(&cost_usds)
    .bind(&subscription_ids)
    .bind(&content_hashes)
    .execute(pool)
    .await;

    match result {
        Ok(_) => tracing::debug!("Flushed {len} token usage log entries"),
        Err(e) => tracing::warn!("Failed to flush {len} token usage log entries: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry() -> TokenUsageEntry {
        TokenUsageEntry {
            group_id: Uuid::new_v4(),
            server_id: Uuid::new_v4(),
            model: Some("claude-opus-4-6".to_string()),
            input_tokens: 100,
            output_tokens: 200,
            cache_creation_tokens: None,
            cache_read_tokens: None,
            is_dynamic_key: false,
            key_hash: None,
            group_key_id: None,
            cost_usd: None,
            subscription_id: None,
            created_at: Utc::now(),
            content_hash: None,
        }
    }

    #[tokio::test]
    async fn test_buffer_overflow_drops_entry() {
        let (tx, _rx) = mpsc::channel(1);
        tx.try_send(make_entry()).unwrap();
        assert!(tx.try_send(make_entry()).is_err());
    }

    #[tokio::test]
    async fn test_buffer_send_succeeds_when_not_full() {
        let (tx, _rx) = mpsc::channel(10_000);
        assert!(tx.try_send(make_entry()).is_ok());
        assert!(tx.try_send(make_entry()).is_ok());
    }

    #[tokio::test]
    async fn test_channel_close_signals_receiver() {
        let (tx, mut rx) = mpsc::channel::<TokenUsageEntry>(10);
        tx.try_send(make_entry()).unwrap();
        drop(tx);
        assert!(rx.recv().await.is_some());
        assert!(rx.recv().await.is_none());
    }

    #[test]
    fn test_hash_key_deterministic() {
        let h1 = hash_key("sk-ant-test-key-123");
        let h2 = hash_key("sk-ant-test-key-123");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16);
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_key_different_inputs() {
        let h1 = hash_key("key-a");
        let h2 = hash_key("key-b");
        assert_ne!(h1, h2);
    }
}
