use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UptimeCheckEntry {
    pub group_id: Uuid,
    pub server_id: Uuid,
    pub status_code: i16,
    pub latency_ms: i32,
    pub request_id: Uuid,
    pub request_model: Option<String>,
    pub created_at: DateTime<Utc>,
}

const BATCH_SIZE: usize = 100;
const FLUSH_INTERVAL_SECS: u64 = 5;

pub async fn flush_task(mut rx: mpsc::Receiver<UptimeCheckEntry>, pool: PgPool) {
    let mut buffer: Vec<UptimeCheckEntry> = Vec::with_capacity(BATCH_SIZE);
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

async fn flush_batch(pool: &PgPool, entries: &[UptimeCheckEntry]) {
    let len = entries.len();
    let mut ids = Vec::with_capacity(len);
    let mut created_ats = Vec::with_capacity(len);
    let mut group_ids = Vec::with_capacity(len);
    let mut server_ids = Vec::with_capacity(len);
    let mut status_codes = Vec::with_capacity(len);
    let mut latency_mss = Vec::with_capacity(len);
    let mut request_ids = Vec::with_capacity(len);
    let mut request_models = Vec::with_capacity(len);

    for e in entries {
        ids.push(Uuid::new_v4());
        created_ats.push(e.created_at);
        group_ids.push(e.group_id);
        server_ids.push(e.server_id);
        status_codes.push(e.status_code);
        latency_mss.push(e.latency_ms);
        request_ids.push(e.request_id);
        request_models.push(e.request_model.as_deref());
    }

    let result = sqlx::query(
        "INSERT INTO uptime_checks \
         (id, created_at, group_id, server_id, status_code, latency_ms, request_id, request_model) \
         SELECT * FROM UNNEST(\
           $1::uuid[], $2::timestamptz[], $3::uuid[], $4::uuid[], \
           $5::smallint[], $6::integer[], $7::uuid[], $8::text[])",
    )
    .bind(&ids)
    .bind(&created_ats)
    .bind(&group_ids)
    .bind(&server_ids)
    .bind(&status_codes)
    .bind(&latency_mss)
    .bind(&request_ids)
    .bind(&request_models)
    .execute(pool)
    .await;

    match result {
        Ok(_) => tracing::debug!("Flushed {len} uptime check entries"),
        Err(e) => tracing::warn!("Failed to flush {len} uptime check entries: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry() -> UptimeCheckEntry {
        UptimeCheckEntry {
            group_id: Uuid::new_v4(),
            server_id: Uuid::new_v4(),
            status_code: 200,
            latency_ms: 150,
            request_id: Uuid::new_v4(),
            request_model: Some("test-model".to_string()),
            created_at: Utc::now(),
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
        let (tx, mut rx) = mpsc::channel::<UptimeCheckEntry>(10);
        tx.try_send(make_entry()).unwrap();
        drop(tx);
        assert!(rx.recv().await.is_some());
        assert!(rx.recv().await.is_none());
    }
}
