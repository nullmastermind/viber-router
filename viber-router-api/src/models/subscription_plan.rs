use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Deserialize a double-option field: missing → None, null → Some(None), value → Some(Some(v))
fn deserialize_optional_nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubscriptionPlan {
    pub id: Uuid,
    pub name: String,
    pub sub_type: String,
    pub cost_limit_usd: f64,
    pub model_limits: serde_json::Value,
    pub model_request_costs: serde_json::Value,
    pub reset_hours: Option<i32>,
    pub duration_days: i32,
    pub rpm_limit: Option<f64>,
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
    pub model_request_costs: Option<serde_json::Value>,
    pub reset_hours: Option<i32>,
    pub duration_days: i32,
    pub rpm_limit: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubscriptionPlan {
    pub name: Option<String>,
    pub sub_type: Option<String>,
    pub cost_limit_usd: Option<f64>,
    pub model_limits: Option<serde_json::Value>,
    pub model_request_costs: Option<serde_json::Value>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    pub reset_hours: Option<Option<i32>>,
    pub duration_days: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    pub rpm_limit: Option<Option<f64>>,
    pub is_active: Option<bool>,
}
