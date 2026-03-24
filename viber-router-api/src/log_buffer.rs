use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverAttempt {
    pub server_id: Uuid,
    pub server_name: String,
    pub status: u16,
    pub latency_ms: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_headers: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ProxyLogEntry {
    pub group_id: Uuid,
    pub group_api_key: String,
    pub server_id: Uuid,
    pub server_name: String,
    pub request_path: String,
    pub request_method: String,
    pub status_code: i16,
    pub error_type: String,
    pub latency_ms: i32,
    pub failover_chain: Vec<FailoverAttempt>,
    pub request_model: Option<String>,
    pub request_body: Option<serde_json::Value>,
    pub request_headers: Option<serde_json::Value>,
    pub upstream_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

const BATCH_SIZE: usize = 100;
const FLUSH_INTERVAL_SECS: u64 = 5;

pub async fn flush_task(mut rx: mpsc::Receiver<ProxyLogEntry>, pool: PgPool) {
    let mut buffer: Vec<ProxyLogEntry> = Vec::with_capacity(BATCH_SIZE);
    let mut interval = tokio::time::interval(
        std::time::Duration::from_secs(FLUSH_INTERVAL_SECS),
    );

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

async fn flush_batch(pool: &PgPool, entries: &[ProxyLogEntry]) {
    let len = entries.len();
    let mut ids = Vec::with_capacity(len);
    let mut created_ats = Vec::with_capacity(len);
    let mut group_ids = Vec::with_capacity(len);
    let mut group_api_keys = Vec::with_capacity(len);
    let mut server_ids = Vec::with_capacity(len);
    let mut server_names = Vec::with_capacity(len);
    let mut request_paths = Vec::with_capacity(len);
    let mut request_methods = Vec::with_capacity(len);
    let mut status_codes: Vec<i16> = Vec::with_capacity(len);
    let mut error_types = Vec::with_capacity(len);
    let mut latency_mss: Vec<i32> = Vec::with_capacity(len);
    let mut failover_chains = Vec::with_capacity(len);
    let mut request_models: Vec<Option<String>> = Vec::with_capacity(len);
    let mut request_bodies: Vec<Option<serde_json::Value>> = Vec::with_capacity(len);
    let mut request_headers_list: Vec<Option<serde_json::Value>> = Vec::with_capacity(len);
    let mut upstream_urls: Vec<Option<String>> = Vec::with_capacity(len);

    for e in entries {
        ids.push(Uuid::new_v4());
        created_ats.push(e.created_at);
        group_ids.push(e.group_id);
        group_api_keys.push(e.group_api_key.clone());
        server_ids.push(e.server_id);
        server_names.push(e.server_name.clone());
        request_paths.push(e.request_path.clone());
        request_methods.push(e.request_method.clone());
        status_codes.push(e.status_code);
        error_types.push(e.error_type.clone());
        latency_mss.push(e.latency_ms);
        failover_chains.push(
            serde_json::to_value(&e.failover_chain).unwrap_or_default(),
        );
        request_models.push(e.request_model.clone());
        request_bodies.push(e.request_body.clone());
        request_headers_list.push(e.request_headers.clone());
        upstream_urls.push(e.upstream_url.clone());
    }

    let result = sqlx::query(
        "INSERT INTO proxy_logs \
         (id, created_at, group_id, group_api_key, server_id, server_name, \
          request_path, request_method, status_code, error_type, latency_ms, \
          failover_chain, request_model, request_body, request_headers, upstream_url) \
         SELECT * FROM UNNEST(\
           $1::uuid[], $2::timestamptz[], $3::uuid[], $4::text[], \
           $5::uuid[], $6::text[], $7::text[], $8::text[], \
           $9::smallint[], $10::text[], $11::integer[], \
           $12::jsonb[], $13::text[], $14::jsonb[], $15::jsonb[], $16::text[])",
    )
    .bind(&ids)
    .bind(&created_ats)
    .bind(&group_ids)
    .bind(&group_api_keys)
    .bind(&server_ids)
    .bind(&server_names)
    .bind(&request_paths)
    .bind(&request_methods)
    .bind(&status_codes)
    .bind(&error_types)
    .bind(&latency_mss)
    .bind(&failover_chains)
    .bind(&request_models)
    .bind(&request_bodies)
    .bind(&request_headers_list)
    .bind(&upstream_urls)
    .execute(pool)
    .await;

    match result {
        Ok(_) => tracing::debug!("Flushed {len} proxy log entries"),
        Err(e) => tracing::warn!("Failed to flush {len} proxy log entries: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry() -> ProxyLogEntry {
        ProxyLogEntry {
            group_id: Uuid::new_v4(),
            group_api_key: "sk-test".to_string(),
            server_id: Uuid::new_v4(),
            server_name: "test-server".to_string(),
            request_path: "/v1/messages".to_string(),
            request_method: "POST".to_string(),
            status_code: 429,
            error_type: "upstream_error".to_string(),
            latency_ms: 100,
            failover_chain: vec![FailoverAttempt {
                server_id: Uuid::new_v4(),
                server_name: "srv-1".to_string(),
                status: 429,
                latency_ms: 100,
                resolved_key: None,
                upstream_url: None,
                request_headers: None,
                request_body: None,
            }],
            request_model: Some("claude-opus-4-6".to_string()),
            request_body: None,
            request_headers: None,
            upstream_url: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_buffer_overflow_drops_entry() {
        let (tx, _rx) = mpsc::channel(1);
        // Fill the channel
        tx.try_send(make_entry()).unwrap();
        // Next send should fail (buffer full)
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
        let (tx, mut rx) = mpsc::channel::<ProxyLogEntry>(10);
        tx.try_send(make_entry()).unwrap();
        drop(tx);
        // Should receive the entry
        assert!(rx.recv().await.is_some());
        // Then None (channel closed)
        assert!(rx.recv().await.is_none());
    }

    #[test]
    fn test_failover_attempt_serialization() {
        let attempt = FailoverAttempt {
            server_id: Uuid::nil(),
            server_name: "srv-1".to_string(),
            status: 429,
            latency_ms: 120,
            resolved_key: None,
            upstream_url: None,
            request_headers: None,
            request_body: None,
        };
        let json = serde_json::to_value(&attempt).unwrap();
        assert_eq!(json["status"], 429);
        assert_eq!(json["server_name"], "srv-1");
        assert_eq!(json["latency_ms"], 120);
    }
}
