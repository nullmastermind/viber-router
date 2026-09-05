//! Admin ping of a group-assigned server: apply model mapping, send a
//! minimal streaming /v1/messages request, and measure TTFT.

use std::time::{Duration, Instant};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::routes::AppState;

const PING_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Serialize)]
pub struct PingResult {
    pub ok: bool,
    pub status: u16,
    pub ttft_ms: Option<u64>,
    pub model: String,
    pub mapped_model: String,
    pub error: Option<String>,
}

fn ping_request_body(model: &str, mappings: &Value) -> Value {
    let mapped = mappings
        .as_object()
        .and_then(|m| m.get(model))
        .and_then(|v| v.as_str())
        .unwrap_or(model);
    serde_json::json!({
        "model": mapped,
        "max_tokens": 1,
        "stream": true,
        "messages": [{"role": "user", "content": "ping"}]
    })
}

fn apply_ping_headers(
    mut req: reqwest::RequestBuilder,
    custom_headers: &Option<Value>,
) -> reqwest::RequestBuilder {
    let Some(obj) = custom_headers.as_ref().and_then(Value::as_object) else {
        return req;
    };
    for (name, value) in obj {
        let Some(val_str) = value.as_str() else {
            continue;
        };
        req = req.header(name.as_str(), val_str);
    }
    req
}

async fn ping_upstream(
    client: &reqwest::Client,
    base_url: &str,
    api_key: Option<&str>,
    custom_headers: &Option<Value>,
    model: &str,
    mappings: &Value,
    timeout: Duration,
) -> PingResult {
    let body = ping_request_body(model, mappings);
    let mapped_model = body["model"].as_str().unwrap_or(model).to_string();
    let url = format!("{}/v1/messages", base_url.trim_end_matches('/'));
    let start = Instant::now();

    let mut req = client
        .post(&url)
        .timeout(timeout)
        .header("content-type", "application/json")
        .header("anthropic-version", "2023-06-01")
        .body(body.to_string());
    if let Some(key) = api_key {
        req = req.header("x-api-key", key);
    }
    req = apply_ping_headers(req, custom_headers);

    match req.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            if !resp.status().is_success() {
                let error = resp.text().await.ok().filter(|s| !s.is_empty());
                return PingResult {
                    ok: false,
                    status,
                    ttft_ms: None,
                    model: model.to_string(),
                    mapped_model,
                    error,
                };
            }
            let mut stream = resp.bytes_stream();
            match stream.next().await {
                Some(Ok(_)) => PingResult {
                    ok: true,
                    status,
                    ttft_ms: Some(start.elapsed().as_millis() as u64),
                    model: model.to_string(),
                    mapped_model,
                    error: None,
                },
                Some(Err(e)) => PingResult {
                    ok: false,
                    status,
                    ttft_ms: None,
                    model: model.to_string(),
                    mapped_model,
                    error: Some(e.to_string()),
                },
                None => PingResult {
                    ok: false,
                    status,
                    ttft_ms: None,
                    model: model.to_string(),
                    mapped_model,
                    error: Some("empty stream".into()),
                },
            }
        }
        Err(e) => PingResult {
            ok: false,
            status: 0,
            ttft_ms: None,
            model: model.to_string(),
            mapped_model,
            error: Some(e.to_string()),
        },
    }
}

fn normalize_ping_model(model: &str) -> Option<String> {
    let trimmed = model.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[derive(Debug, Deserialize)]
pub struct PingRequest {
    pub model: String,
}

type ApiError = (StatusCode, Json<Value>);

fn err(status: StatusCode, msg: &str) -> ApiError {
    (status, Json(serde_json::json!({"error": msg})))
}

pub async fn ping_server(
    State(state): State<AppState>,
    Path((group_id, server_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<PingRequest>,
) -> Result<Json<PingResult>, ApiError> {
    let model = normalize_ping_model(&input.model)
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "Model is required"))?;

    let row = sqlx::query_as::<_, (String, Option<String>, Option<Value>, Value)>(
        "SELECT s.base_url, s.api_key, s.custom_headers, gs.model_mappings \
         FROM group_servers gs JOIN servers s ON s.id = gs.server_id \
         WHERE gs.group_id = $1 AND gs.server_id = $2",
    )
    .bind(group_id)
    .bind(server_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let Some((base_url, api_key, custom_headers, model_mappings)) = row else {
        return Err(err(StatusCode::NOT_FOUND, "Assignment not found"));
    };

    let result = ping_upstream(
        &state.http_client,
        &base_url,
        api_key.as_deref(),
        &custom_headers,
        &model,
        &model_mappings,
        PING_TIMEOUT,
    )
    .await;

    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn blank_model_is_rejected() {
        assert!(normalize_ping_model("").is_none());
        assert!(normalize_ping_model("   ").is_none());
        assert_eq!(
            normalize_ping_model("  claude-sonnet-4-6  ").as_deref(),
            Some("claude-sonnet-4-6")
        );
    }

    #[test]
    fn ping_body_uses_mapped_model_when_mapping_exists() {
        let mappings = json!({"claude-sonnet-4-6": "my-sonnet"});
        let body = ping_request_body("claude-sonnet-4-6", &mappings);
        assert_eq!(body["model"], "my-sonnet");
    }

    #[test]
    fn ping_body_is_minimal_streaming_messages_request() {
        let body = ping_request_body("claude-sonnet-4-6", &json!({}));
        assert_eq!(body["max_tokens"], 1);
        assert_eq!(body["stream"], true);
        assert_eq!(
            body["messages"],
            json!([{"role": "user", "content": "ping"}])
        );
    }

    #[test]
    fn ping_body_keeps_unmapped_model_name() {
        let mappings = json!({"other-model": "mapped"});
        let body = ping_request_body("claude-sonnet-4-6", &mappings);
        assert_eq!(body["model"], "claude-sonnet-4-6");
    }

    #[tokio::test]
    async fn ping_upstream_sends_mapped_model_and_reports_ttft() {
        use axum::Json;
        use axum::Router;
        use axum::http::header;
        use axum::routing::post;
        use std::sync::{Arc, Mutex};
        use std::time::Duration;

        let captured: Arc<Mutex<Option<Value>>> = Arc::new(Mutex::new(None));
        let captured_for_handler = captured.clone();

        let app = Router::new().route(
            "/v1/messages",
            post(move |Json(body): Json<Value>| {
                let captured_for_handler = captured_for_handler.clone();
                async move {
                    *captured_for_handler.lock().unwrap() = Some(body);
                    (
                        [(header::CONTENT_TYPE, "text/event-stream")],
                        "event: message_start\ndata: {\"type\":\"message_start\"}\n\n",
                    )
                }
            }),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let result = ping_upstream(
            &reqwest::Client::new(),
            &format!("http://{addr}"),
            Some("sk-test"),
            &None,
            "claude-sonnet-4-6",
            &json!({"claude-sonnet-4-6": "my-sonnet"}),
            Duration::from_secs(5),
        )
        .await;

        assert!(result.ok, "expected live server, got {:?}", result.error);
        assert_eq!(result.status, 200);
        assert!(result.ttft_ms.is_some(), "TTFT should be measured");
        assert_eq!(result.model, "claude-sonnet-4-6");
        assert_eq!(result.mapped_model, "my-sonnet");
        let sent = captured.lock().unwrap().clone().expect("upstream got a body");
        assert_eq!(sent["model"], "my-sonnet");
        assert_eq!(sent["stream"], true);
    }

    #[tokio::test]
    async fn ping_upstream_reports_http_error_without_claiming_alive() {
        use axum::Router;
        use axum::http::StatusCode;
        use axum::routing::post;
        use std::time::Duration;

        let app = Router::new().route(
            "/v1/messages",
            post(|| async { (StatusCode::TOO_MANY_REQUESTS, "rate limited") }),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let result = ping_upstream(
            &reqwest::Client::new(),
            &format!("http://{addr}"),
            Some("sk-test"),
            &None,
            "claude-sonnet-4-6",
            &json!({}),
            Duration::from_secs(5),
        )
        .await;

        assert!(!result.ok);
        assert_eq!(result.status, 429);
        assert!(result.ttft_ms.is_none());
        assert_eq!(result.error.as_deref(), Some("rate limited"));
    }

    #[tokio::test]
    async fn ping_upstream_forwards_api_key_and_custom_headers() {
        use axum::Router;
        use axum::http::{HeaderMap, header};
        use axum::routing::post;
        use std::sync::{Arc, Mutex};
        use std::time::Duration;

        let captured: Arc<Mutex<Option<HeaderMap>>> = Arc::new(Mutex::new(None));
        let captured_for_handler = captured.clone();

        let app = Router::new().route(
            "/v1/messages",
            post(move |headers: HeaderMap| {
                let captured_for_handler = captured_for_handler.clone();
                async move {
                    *captured_for_handler.lock().unwrap() = Some(headers);
                    (
                        [(header::CONTENT_TYPE, "text/event-stream")],
                        "event: message_start\ndata: {}\n\n",
                    )
                }
            }),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let result = ping_upstream(
            &reqwest::Client::new(),
            &format!("http://{addr}"),
            Some("sk-test"),
            &Some(json!({"anthropic-workspace-id": "ws-1"})),
            "claude-sonnet-4-6",
            &json!({}),
            Duration::from_secs(5),
        )
        .await;

        assert!(result.ok, "expected live server, got {:?}", result.error);
        let headers = captured.lock().unwrap().clone().expect("upstream got headers");
        assert_eq!(
            headers.get("x-api-key").and_then(|v| v.to_str().ok()),
            Some("sk-test")
        );
        assert_eq!(
            headers
                .get("anthropic-workspace-id")
                .and_then(|v| v.to_str().ok()),
            Some("ws-1")
        );
        assert_eq!(
            headers
                .get("anthropic-version")
                .and_then(|v| v.to_str().ok()),
            Some("2023-06-01")
        );
    }
}
