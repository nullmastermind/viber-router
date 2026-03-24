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
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateServer {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateServer {
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<Option<String>>,
}
