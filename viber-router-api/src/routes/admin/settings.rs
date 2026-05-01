use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cache;
use crate::models::Settings;
use crate::routes::AppState;

fn default_settings() -> Settings {
    Settings {
        telegram_bot_token: None,
        telegram_chat_ids: vec![],
        alert_status_codes: vec![500, 502, 503],
        alert_cooldown_mins: 5,
        blocked_paths: vec![],
        ct_always_estimate: false,
        ct_anthropic_base_url: None,
        ct_anthropic_api_key: None,
    }
}

async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<Settings>, (StatusCode, Json<Value>)> {
    let row = sqlx::query_as::<_, Settings>(
        "SELECT telegram_bot_token, telegram_chat_ids, alert_status_codes, alert_cooldown_mins, blocked_paths, \
         ct_always_estimate, ct_anthropic_base_url, ct_anthropic_api_key \
         FROM settings WHERE id = 1",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    Ok(Json(row.unwrap_or_else(default_settings)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettings {
    pub telegram_bot_token: Option<Option<String>>,
    pub telegram_chat_ids: Option<Vec<String>>,
    pub alert_status_codes: Option<Vec<i32>>,
    pub alert_cooldown_mins: Option<i32>,
    pub blocked_paths: Option<Vec<String>>,
    pub ct_always_estimate: Option<bool>,
    pub ct_anthropic_base_url: Option<Option<String>>,
    pub ct_anthropic_api_key: Option<Option<String>>,
}

async fn put_settings(
    State(state): State<AppState>,
    Json(input): Json<UpdateSettings>,
) -> Result<Json<Settings>, (StatusCode, Json<Value>)> {
    // Fetch current (or defaults) to merge with partial update
    let current = sqlx::query_as::<_, Settings>(
        "SELECT telegram_bot_token, telegram_chat_ids, alert_status_codes, alert_cooldown_mins, blocked_paths, \
         ct_always_estimate, ct_anthropic_base_url, ct_anthropic_api_key \
         FROM settings WHERE id = 1",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?
    .unwrap_or_else(default_settings);

    let new_token = match input.telegram_bot_token {
        Some(v) => v,
        None => current.telegram_bot_token,
    };
    let new_chat_ids = input.telegram_chat_ids.unwrap_or(current.telegram_chat_ids);
    let new_status_codes = input
        .alert_status_codes
        .unwrap_or(current.alert_status_codes);
    let new_cooldown = input
        .alert_cooldown_mins
        .unwrap_or(current.alert_cooldown_mins);
    let blocked_paths_changed = input.blocked_paths.is_some();
    let new_blocked_paths = input.blocked_paths.unwrap_or(current.blocked_paths);
    let new_ct_always_estimate = input
        .ct_always_estimate
        .unwrap_or(current.ct_always_estimate);
    let new_ct_anthropic_base_url = match input.ct_anthropic_base_url {
        Some(v) => v,
        None => current.ct_anthropic_base_url,
    };
    let new_ct_anthropic_api_key = match input.ct_anthropic_api_key {
        Some(v) => v,
        None => current.ct_anthropic_api_key,
    };

    let updated = sqlx::query_as::<_, Settings>(
        "INSERT INTO settings (id, telegram_bot_token, telegram_chat_ids, alert_status_codes, alert_cooldown_mins, blocked_paths, \
         ct_always_estimate, ct_anthropic_base_url, ct_anthropic_api_key) \
         VALUES (1, $1, $2, $3, $4, $5, $6, $7, $8) \
         ON CONFLICT (id) DO UPDATE SET \
           telegram_bot_token = EXCLUDED.telegram_bot_token, \
           telegram_chat_ids = EXCLUDED.telegram_chat_ids, \
           alert_status_codes = EXCLUDED.alert_status_codes, \
           alert_cooldown_mins = EXCLUDED.alert_cooldown_mins, \
           blocked_paths = EXCLUDED.blocked_paths, \
           ct_always_estimate = EXCLUDED.ct_always_estimate, \
           ct_anthropic_base_url = EXCLUDED.ct_anthropic_base_url, \
           ct_anthropic_api_key = EXCLUDED.ct_anthropic_api_key \
         RETURNING telegram_bot_token, telegram_chat_ids, alert_status_codes, alert_cooldown_mins, blocked_paths, \
         ct_always_estimate, ct_anthropic_base_url, ct_anthropic_api_key",
    )
    .bind(&new_token)
    .bind(&new_chat_ids)
    .bind(&new_status_codes)
    .bind(new_cooldown)
    .bind(&new_blocked_paths)
    .bind(new_ct_always_estimate)
    .bind(&new_ct_anthropic_base_url)
    .bind(&new_ct_anthropic_api_key)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    if blocked_paths_changed {
        cache::invalidate_blocked_paths(&state.redis).await;
    }

    Ok(Json(updated))
}

async fn post_test_alert(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let settings = sqlx::query_as::<_, Settings>(
        "SELECT telegram_bot_token, telegram_chat_ids, alert_status_codes, alert_cooldown_mins, blocked_paths, \
         ct_always_estimate, ct_anthropic_base_url, ct_anthropic_api_key \
         FROM settings WHERE id = 1",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?
    .unwrap_or_else(default_settings);

    let token = settings.telegram_bot_token.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Bot token not configured"})),
        )
    })?;

    if settings.telegram_chat_ids.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "No chat IDs configured"})),
        ));
    }

    let text = "🔔 *Test Alert*\nViber Router Telegram alerts are configured correctly\\.";
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);

    for chat_id in &settings.telegram_chat_ids {
        let resp = state
            .http_client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": text,
                "parse_mode": "MarkdownV2"
            }))
            .send()
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
            })?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err((
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": err_text})),
            ));
        }
    }

    Ok(Json(serde_json::json!({"success": true})))
}

#[derive(Debug, Serialize)]
struct TelegramChat {
    chat_id: String,
    first_name: Option<String>,
    username: Option<String>,
}

async fn get_telegram_chats(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let settings = sqlx::query_as::<_, Settings>(
        "SELECT telegram_bot_token, telegram_chat_ids, alert_status_codes, alert_cooldown_mins, blocked_paths, \
         ct_always_estimate, ct_anthropic_base_url, ct_anthropic_api_key \
         FROM settings WHERE id = 1",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?
    .unwrap_or_else(default_settings);

    let token = settings.telegram_bot_token.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Bot token not configured"})),
        )
    })?;

    let url = format!("https://api.telegram.org/bot{}/getUpdates?limit=100", token);

    let resp = state.http_client.get(&url).send().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": err_text})),
        ));
    }

    let body: Value = resp.json().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    let mut seen_ids = std::collections::HashSet::new();
    let mut chats: Vec<TelegramChat> = Vec::new();

    if let Some(results) = body.get("result").and_then(|r| r.as_array()) {
        for update in results {
            // Check message.chat or my_chat_member.chat etc.
            let chat = update
                .get("message")
                .and_then(|m| m.get("chat"))
                .or_else(|| update.get("channel_post").and_then(|m| m.get("chat")));

            if let Some(chat) = chat {
                let chat_id = match chat.get("id").and_then(|id| id.as_i64()) {
                    Some(id) => id.to_string(),
                    None => continue,
                };
                if seen_ids.insert(chat_id.clone()) {
                    chats.push(TelegramChat {
                        chat_id,
                        first_name: chat
                            .get("first_name")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        username: chat
                            .get("username")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                    });
                }
            }
        }
    }

    Ok(Json(serde_json::json!({"chats": chats})))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_settings).put(put_settings))
        .route("/test", post(post_test_alert))
        .route("/telegram-chats", get(get_telegram_chats))
}
