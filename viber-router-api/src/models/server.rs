use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Server {
    pub id: Uuid,
    pub short_id: i32,
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub password_hash: Option<String>,
    pub system_prompt: Option<String>,
    pub remove_thinking: bool,
    pub custom_headers: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateServer {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub password: Option<String>,
    pub system_prompt: Option<String>,
    pub remove_thinking: Option<bool>,
    pub custom_headers: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateServer {
    pub name: Option<String>,
    pub base_url: Option<String>,
    #[serde(default, deserialize_with = "crate::serde_utils::double_option")]
    pub api_key: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::serde_utils::double_option")]
    pub password: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::serde_utils::double_option")]
    pub system_prompt: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::serde_utils::double_option")]
    pub remove_thinking: Option<Option<bool>>,
    #[serde(default, deserialize_with = "crate::serde_utils::double_option")]
    pub custom_headers: Option<Option<serde_json::Value>>,
}
