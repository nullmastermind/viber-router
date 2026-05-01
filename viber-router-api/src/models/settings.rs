use serde::{Deserialize, Serialize};
use sqlx::FromRow;

fn default_timezone() -> String {
    "Asia/Ho_Chi_Minh".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Settings {
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_ids: Vec<String>,
    pub alert_status_codes: Vec<i32>,
    pub alert_cooldown_mins: i32,
    #[serde(default)]
    pub blocked_paths: Vec<String>,
    #[serde(default = "default_timezone")]
    pub timezone: String,
    #[serde(default)]
    pub ct_always_estimate: bool,
    pub ct_anthropic_base_url: Option<String>,
    pub ct_anthropic_api_key: Option<String>,
}
