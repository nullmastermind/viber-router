use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubscriptionPlan {
    pub id: Uuid,
    pub name: String,
    pub sub_type: String,
    pub cost_limit_usd: f64,
    pub model_limits: serde_json::Value,
    pub reset_hours: Option<i32>,
    pub duration_days: i32,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionPlan {
    pub name: String,
    pub sub_type: String,
    pub cost_limit_usd: f64,
    pub model_limits: Option<serde_json::Value>,
    pub reset_hours: Option<i32>,
    pub duration_days: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubscriptionPlan {
    pub name: Option<String>,
    pub sub_type: Option<String>,
    pub cost_limit_usd: Option<f64>,
    pub model_limits: Option<serde_json::Value>,
    pub reset_hours: Option<Option<i32>>,
    pub duration_days: Option<i32>,
    pub is_active: Option<bool>,
}
