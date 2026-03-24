use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupServer {
    pub group_id: Uuid,
    pub server_id: Uuid,
    pub priority: i32,
    pub model_mappings: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupServerDetail {
    pub server_id: Uuid,
    pub short_id: i32,
    pub server_name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub priority: i32,
    pub model_mappings: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct AssignServer {
    pub server_id: Uuid,
    pub priority: i32,
    pub model_mappings: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssignment {
    pub priority: Option<i32>,
    pub model_mappings: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderServers {
    pub server_ids: Vec<Uuid>,
}

/// Full config used by the proxy engine (cached in Redis)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    pub group_id: Uuid,
    pub api_key: String,
    pub is_active: bool,
    pub failover_status_codes: Vec<u16>,
    pub servers: Vec<GroupServerDetail>,
}
