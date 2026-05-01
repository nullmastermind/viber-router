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
    pub weekly_cost_limit_usd: Option<f64>,
    pub model_limits: serde_json::Value,
    pub model_request_costs: serde_json::Value,
    pub reset_hours: Option<i32>,
    pub duration_days: i32,
    pub rpm_limit: Option<f64>,
    pub tpm_limit: Option<f64>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionPlan {
    pub name: String,
    pub sub_type: String,
    pub cost_limit_usd: f64,
    pub weekly_cost_limit_usd: Option<f64>,
    pub model_limits: Option<serde_json::Value>,
    pub model_request_costs: Option<serde_json::Value>,
    pub reset_hours: Option<i32>,
    pub duration_days: i32,
    pub rpm_limit: Option<f64>,
    pub tpm_limit: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubscriptionPlan {
    pub name: Option<String>,
    pub sub_type: Option<String>,
    pub cost_limit_usd: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    pub weekly_cost_limit_usd: Option<Option<f64>>,
    pub model_limits: Option<serde_json::Value>,
    pub model_request_costs: Option<serde_json::Value>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    pub reset_hours: Option<Option<i32>>,
    pub duration_days: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    pub rpm_limit: Option<Option<f64>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    pub tpm_limit: Option<Option<f64>>,
    pub is_active: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_subscription_plan_deserializes_tpm_limit() {
        let plan: CreateSubscriptionPlan = serde_json::from_value(serde_json::json!({
            "name": "TPM Plan",
            "sub_type": "fixed",
            "cost_limit_usd": 10.0,
            "duration_days": 30,
            "tpm_limit": 100000.0
        }))
        .unwrap();

        assert_eq!(plan.tpm_limit, Some(100000.0));
        assert_eq!(plan.weekly_cost_limit_usd, None);
    }

    #[test]
    fn create_subscription_plan_deserializes_missing_tpm_as_unlimited() {
        let plan: CreateSubscriptionPlan = serde_json::from_value(serde_json::json!({
            "name": "Unlimited Plan",
            "sub_type": "fixed",
            "cost_limit_usd": 10.0,
            "duration_days": 30
        }))
        .unwrap();

        assert_eq!(plan.tpm_limit, None);
        assert_eq!(plan.weekly_cost_limit_usd, None);
    }

    #[test]
    fn update_subscription_plan_distinguishes_missing_clear_and_set_tpm() {
        let missing: UpdateSubscriptionPlan =
            serde_json::from_value(serde_json::json!({})).unwrap();
        assert_eq!(missing.tpm_limit, None);
        assert_eq!(missing.weekly_cost_limit_usd, None);

        let clear: UpdateSubscriptionPlan = serde_json::from_value(serde_json::json!({
            "tpm_limit": null
        }))
        .unwrap();
        assert_eq!(clear.tpm_limit, Some(None));

        let set: UpdateSubscriptionPlan = serde_json::from_value(serde_json::json!({
            "tpm_limit": 120000.0
        }))
        .unwrap();
        assert_eq!(set.tpm_limit, Some(Some(120000.0)));
    }
}
