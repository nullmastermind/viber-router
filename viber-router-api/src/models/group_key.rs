use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupKey {
    pub id: Uuid,
    pub group_id: Uuid,
    pub api_key: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupKey {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupKey {
    pub name: Option<String>,
    pub is_active: Option<bool>,
}
