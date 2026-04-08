use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KeySubscription {
    pub id: Uuid,
    pub group_key_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub sub_type: String,
    pub cost_limit_usd: f64,
    pub model_limits: serde_json::Value,
    pub model_request_costs: serde_json::Value,
    pub reset_hours: Option<i32>,
    pub duration_days: i32,
    pub rpm_limit: Option<f64>,
    pub status: String,
    pub activated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AssignSubscription {
    pub plan_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CancelSubscription {
    pub status: String,
}
