use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupServer {
    pub group_id: Uuid,
    pub server_id: Uuid,
    pub priority: i32,
    pub model_mappings: serde_json::Value,
    pub is_enabled: bool,
    pub cb_max_failures: Option<i32>,
    pub cb_window_seconds: Option<i32>,
    pub cb_cooldown_seconds: Option<i32>,
    pub rate_input: Option<f64>,
    pub rate_output: Option<f64>,
    pub rate_cache_write: Option<f64>,
    pub rate_cache_read: Option<f64>,
    pub max_requests: Option<i32>,
    pub rate_window_seconds: Option<i32>,
    pub normalize_cache_read: bool,
    pub max_input_tokens: Option<i32>,
    pub min_input_tokens: Option<i32>,
    pub supported_models: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[sqlx(default)]
    #[serde(default)]
    pub active_hours_start: Option<String>,
    #[sqlx(default)]
    #[serde(default)]
    pub active_hours_end: Option<String>,
    #[sqlx(default)]
    #[serde(default)]
    pub active_hours_timezone: Option<String>,
    #[sqlx(default)]
    #[serde(default)]
    pub retry_status_codes: Option<Vec<i32>>,
    #[sqlx(default)]
    #[serde(default)]
    pub retry_count: Option<i32>,
    #[sqlx(default)]
    #[serde(default)]
    pub retry_delay_seconds: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupServerDetail {
    pub server_id: Uuid,
    pub short_id: i32,
    pub server_name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub remove_thinking: bool,
    pub priority: i32,
    pub model_mappings: serde_json::Value,
    pub is_enabled: bool,
    pub cb_max_failures: Option<i32>,
    pub cb_window_seconds: Option<i32>,
    pub cb_cooldown_seconds: Option<i32>,
    pub rate_input: Option<f64>,
    pub rate_output: Option<f64>,
    pub rate_cache_write: Option<f64>,
    pub rate_cache_read: Option<f64>,
    pub max_requests: Option<i32>,
    pub rate_window_seconds: Option<i32>,
    #[serde(default)]
    pub normalize_cache_read: bool,
    #[serde(default)]
    pub max_input_tokens: Option<i32>,
    #[serde(default)]
    pub min_input_tokens: Option<i32>,
    #[serde(default)]
    pub supported_models: Vec<String>,
    #[sqlx(default)]
    #[serde(default)]
    pub active_hours_start: Option<String>,
    #[sqlx(default)]
    #[serde(default)]
    pub active_hours_end: Option<String>,
    #[sqlx(default)]
    #[serde(default)]
    pub active_hours_timezone: Option<String>,
    #[sqlx(default)]
    #[serde(default)]
    pub retry_status_codes: Option<Vec<i32>>,
    #[sqlx(default)]
    #[serde(default)]
    pub retry_count: Option<i32>,
    #[sqlx(default)]
    #[serde(default)]
    pub retry_delay_seconds: Option<f64>,
}

/// Admin-facing server detail with rate fields (not used in proxy cache)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminGroupServerDetail {
    pub server_id: Uuid,
    pub short_id: i32,
    pub server_name: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub priority: i32,
    pub model_mappings: serde_json::Value,
    pub is_enabled: bool,
    pub cb_max_failures: Option<i32>,
    pub cb_window_seconds: Option<i32>,
    pub cb_cooldown_seconds: Option<i32>,
    pub rate_input: Option<f64>,
    pub rate_output: Option<f64>,
    pub rate_cache_write: Option<f64>,
    pub rate_cache_read: Option<f64>,
    pub max_requests: Option<i32>,
    pub rate_window_seconds: Option<i32>,
    pub normalize_cache_read: bool,
    pub max_input_tokens: Option<i32>,
    pub min_input_tokens: Option<i32>,
    #[serde(default)]
    pub supported_models: Vec<String>,
    pub password_hash: Option<String>,
    #[sqlx(default)]
    #[serde(default)]
    pub active_hours_start: Option<String>,
    #[sqlx(default)]
    #[serde(default)]
    pub active_hours_end: Option<String>,
    #[sqlx(default)]
    #[serde(default)]
    pub active_hours_timezone: Option<String>,
    #[sqlx(default)]
    #[serde(default)]
    pub retry_status_codes: Option<Vec<i32>>,
    #[sqlx(default)]
    #[serde(default)]
    pub retry_count: Option<i32>,
    #[sqlx(default)]
    #[serde(default)]
    pub retry_delay_seconds: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct AssignServer {
    pub server_id: Uuid,
    pub priority: i32,
    pub model_mappings: Option<serde_json::Value>,
    pub max_input_tokens: Option<i32>,
    pub min_input_tokens: Option<i32>,
    #[serde(default)]
    pub supported_models: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssignment {
    pub priority: Option<i32>,
    pub model_mappings: Option<serde_json::Value>,
    pub is_enabled: Option<bool>,
    pub cb_max_failures: Option<Option<i32>>,
    pub cb_window_seconds: Option<Option<i32>>,
    pub cb_cooldown_seconds: Option<Option<i32>>,
    pub rate_input: Option<Option<f64>>,
    pub rate_output: Option<Option<f64>>,
    pub rate_cache_write: Option<Option<f64>>,
    pub rate_cache_read: Option<Option<f64>>,
    pub max_requests: Option<Option<i32>>,
    pub rate_window_seconds: Option<Option<i32>>,
    pub normalize_cache_read: Option<bool>,
    pub max_input_tokens: Option<Option<i32>>,
    pub min_input_tokens: Option<Option<i32>>,
    pub supported_models: Option<Vec<String>>,
    pub active_hours_start: Option<Option<String>>,
    pub active_hours_end: Option<Option<String>>,
    pub active_hours_timezone: Option<Option<String>>,
    pub retry_status_codes: Option<Option<Vec<i32>>>,
    pub retry_count: Option<Option<i32>>,
    pub retry_delay_seconds: Option<Option<f64>>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderServers {
    pub server_ids: Vec<Uuid>,
}

/// Count-tokens default server detail (embedded in GroupConfig)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountTokensServer {
    pub server_id: Uuid,
    pub short_id: i32,
    pub server_name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub system_prompt: Option<String>,
    pub remove_thinking: bool,
    pub model_mappings: serde_json::Value,
}

/// Full config used by the proxy engine (cached in Redis)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    pub group_id: Uuid,
    pub group_name: String,
    pub api_key: String,
    pub is_active: bool,
    pub failover_status_codes: Vec<u16>,
    pub ttft_timeout_ms: Option<i32>,
    pub servers: Vec<GroupServerDetail>,
    pub count_tokens_server: Option<CountTokensServer>,
    pub group_key_id: Option<Uuid>,
    #[serde(default)]
    pub allowed_models: Vec<String>,
    #[serde(default)]
    pub key_allowed_models: Vec<String>,
    #[serde(default)]
    pub blocked_user_agents: Vec<String>,
}
