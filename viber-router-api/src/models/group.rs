use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub api_key: String,
    pub failover_status_codes: serde_json::Value,
    pub is_active: bool,
    pub ttft_timeout_ms: Option<i32>,
    pub count_tokens_server_id: Option<Uuid>,
    pub count_tokens_model_mappings: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupListItem {
    pub id: Uuid,
    pub name: String,
    pub api_key: String,
    pub failover_status_codes: serde_json::Value,
    pub is_active: bool,
    pub ttft_timeout_ms: Option<i32>,
    pub count_tokens_server_id: Option<Uuid>,
    pub count_tokens_model_mappings: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub servers_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroup {
    pub name: String,
    pub failover_status_codes: Option<Vec<u16>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroup {
    pub name: Option<String>,
    pub failover_status_codes: Option<Vec<u16>>,
    pub is_active: Option<bool>,
    pub ttft_timeout_ms: Option<Option<i32>>,
    pub count_tokens_server_id: Option<Option<Uuid>>,
    pub count_tokens_model_mappings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupWithServers {
    #[serde(flatten)]
    pub group: Group,
    pub servers: Vec<super::AdminGroupServerDetail>,
    pub allowed_models: Vec<super::Model>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub total_pages: i64,
}

pub fn generate_api_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();
    let suffix: String = (0..24)
        .map(|_| {
            let idx = rng.random_range(..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("sk-vibervn-{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_has_correct_prefix() {
        let key = generate_api_key();
        assert!(key.starts_with("sk-vibervn-"));
    }

    #[test]
    fn test_api_key_has_correct_length() {
        let key = generate_api_key();
        // "sk-vibervn-" (11 chars) + 24 random chars = 35
        assert_eq!(key.len(), 35);
    }

    #[test]
    fn test_api_key_is_alphanumeric_suffix() {
        let key = generate_api_key();
        let suffix = &key["sk-vibervn-".len()..];
        assert!(suffix.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_api_keys_are_unique() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();
        assert_ne!(key1, key2);
    }
}
