use axum::{
    Router,
    body::Body,
    extract::{OriginalUri, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use serde_json::Value;

use crate::cache;
use crate::models::{GroupConfig, GroupServerDetail};
use crate::routes::AppState;

pub fn router() -> Router<AppState> {
    Router::new().fallback(proxy_handler)
}

fn anthropic_error(status: StatusCode, error_type: &str, message: &str) -> Response {
    let body = serde_json::json!({
        "type": "error",
        "error": {
            "type": error_type,
            "message": message
        }
    });
    (status, axum::Json(body)).into_response()
}

async fn resolve_group_config(state: &AppState, api_key: &str) -> Option<GroupConfig> {
    // Try cache first
    if let Some(config) = cache::get_group_config(&state.redis, api_key).await {
        return Some(config);
    }

    // Fall back to DB
    let group = sqlx::query_as::<_, crate::models::Group>(
        "SELECT * FROM groups WHERE api_key = $1",
    )
    .bind(api_key)
    .fetch_optional(&state.db)
    .await
    .ok()??;

    let servers = sqlx::query_as::<_, GroupServerDetail>(
        "SELECT gs.server_id, s.name as server_name, s.base_url, s.api_key, gs.priority, gs.model_mappings \
         FROM group_servers gs JOIN servers s ON s.id = gs.server_id \
         WHERE gs.group_id = $1 ORDER BY gs.priority",
    )
    .bind(group.id)
    .fetch_all(&state.db)
    .await
    .ok()?;

    let failover_codes: Vec<u16> = serde_json::from_value(group.failover_status_codes.clone())
        .unwrap_or_else(|_| vec![429, 500, 502, 503]);

    let config = GroupConfig {
        group_id: group.id,
        api_key: group.api_key.clone(),
        is_active: group.is_active,
        failover_status_codes: failover_codes,
        servers,
    };

    // Cache it for next time
    cache::set_group_config(&state.redis, api_key, &config).await;
    Some(config)
}

fn transform_model(body: &[u8], mappings: &serde_json::Value) -> Vec<u8> {
    let mappings_obj = match mappings.as_object() {
        Some(m) if !m.is_empty() => m,
        _ => return body.to_vec(),
    };

    let mut json: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return body.to_vec(),
    };

    if let Some(model) = json.get("model").and_then(|m| m.as_str())
        && let Some(mapped) = mappings_obj.get(model).and_then(|v| v.as_str())
    {
        json["model"] = Value::String(mapped.to_string());
    }

    serde_json::to_vec(&json).unwrap_or_else(|_| body.to_vec())
}

async fn proxy_handler(
    State(state): State<AppState>,
    OriginalUri(original_uri): OriginalUri,
    req: Request,
) -> Response {
    // Extract API key from x-api-key header
    let api_key = match req.headers().get("x-api-key").and_then(|v| v.to_str().ok()) {
        Some(key) => key.to_string(),
        None => {
            return anthropic_error(
                StatusCode::UNAUTHORIZED,
                "authentication_error",
                "Invalid API key",
            );
        }
    };

    // Look up group config
    let config = match resolve_group_config(&state, &api_key).await {
        Some(c) => c,
        None => {
            return anthropic_error(
                StatusCode::UNAUTHORIZED,
                "authentication_error",
                "Invalid API key",
            );
        }
    };

    if !config.is_active {
        return anthropic_error(
            StatusCode::FORBIDDEN,
            "permission_error",
            "API key is disabled",
        );
    }

    if config.servers.is_empty() {
        return anthropic_error(
            StatusCode::from_u16(429).unwrap(),
            "overloaded_error",
            "All upstream servers unavailable",
        );
    }

    // Capture request parts
    let method = req.method().clone();
    let headers = req.headers().clone();
    let body_bytes = match axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => {
            return anthropic_error(
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                "Failed to read request body",
            );
        }
    };

    let client = &state.http_client;

    // Failover waterfall
    for server in &config.servers {
        let transformed_body = transform_model(&body_bytes, &server.model_mappings);

        // Build upstream URL: server.base_url + original path + query
        let path = original_uri.path();
        let upstream_url = if let Some(query) = original_uri.query() {
            format!("{}{path}?{query}", server.base_url.trim_end_matches('/'))
        } else {
            format!("{}{path}", server.base_url.trim_end_matches('/'))
        };

        // Build upstream request
        let mut upstream_req = client.request(method.clone(), &upstream_url);

        // Forward headers, replacing x-api-key with upstream server's key
        for (name, value) in headers.iter() {
            if name == "x-api-key" {
                continue;
            }
            if name == "host" || name == "content-length" {
                continue;
            }
            if let Ok(reqwest_name) = reqwest::header::HeaderName::from_bytes(name.as_str().as_bytes())
                && let Ok(reqwest_value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes())
            {
                upstream_req = upstream_req.header(reqwest_name, reqwest_value);
            }
        }
        upstream_req = upstream_req.header("x-api-key", &server.api_key);
        upstream_req = upstream_req.body(transformed_body);

        let upstream_resp = match upstream_req.send().await {
            Ok(resp) => resp,
            Err(_) => continue, // Connection error → try next server
        };

        let status = upstream_resp.status().as_u16();

        // Check if this is a failover status code
        if config.failover_status_codes.contains(&status) {
            continue;
        }

        // Success or non-failover error — return this response
        return build_response(upstream_resp).await;
    }

    // All servers exhausted
    let mut resp = anthropic_error(
        StatusCode::TOO_MANY_REQUESTS,
        "overloaded_error",
        "All upstream servers unavailable",
    );
    resp.headers_mut().insert(
        header::HeaderName::from_static("retry-after"),
        HeaderValue::from_static("30"),
    );
    resp
}

async fn build_response(upstream: reqwest::Response) -> Response {
    let status = StatusCode::from_u16(upstream.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let mut response_headers = HeaderMap::new();
    for (name, value) in upstream.headers().iter() {
        if let Ok(axum_name) = axum::http::header::HeaderName::from_bytes(name.as_str().as_bytes())
            && let Ok(axum_value) = HeaderValue::from_bytes(value.as_bytes())
        {
            response_headers.insert(axum_name, axum_value);
        }
    }

    // Check if this is a streaming SSE response
    let is_sse = upstream
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.contains("text/event-stream"));

    if is_sse {
        let stream = upstream.bytes_stream().map(|chunk| {
            chunk
                .map_err(std::io::Error::other)
        });
        let body = Body::from_stream(stream);
        let mut resp = Response::builder().status(status);
        *resp.headers_mut().unwrap() = response_headers;
        resp.body(body).unwrap_or_else(|_| {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })
    } else {
        let body_bytes = upstream.bytes().await.unwrap_or_default();
        let mut resp = Response::builder().status(status);
        *resp.headers_mut().unwrap() = response_headers;
        resp.body(Body::from(body_bytes)).unwrap_or_else(|_| {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_model_with_mapping() {
        let body = br#"{"model":"claude-opus-4-6","messages":[]}"#;
        let mappings = serde_json::json!({"claude-opus-4-6": "my-opus"});
        let result = transform_model(body, &mappings);
        let parsed: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(parsed["model"], "my-opus");
    }

    #[test]
    fn test_transform_model_no_mapping() {
        let body = br#"{"model":"claude-haiku-4-5","messages":[]}"#;
        let mappings = serde_json::json!({"claude-opus-4-6": "my-opus"});
        let result = transform_model(body, &mappings);
        let parsed: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(parsed["model"], "claude-haiku-4-5");
    }

    #[test]
    fn test_transform_model_empty_mappings() {
        let body = br#"{"model":"claude-opus-4-6","messages":[]}"#;
        let mappings = serde_json::json!({});
        let result = transform_model(body, &mappings);
        assert_eq!(result, body);
    }

    #[test]
    fn test_transform_model_no_model_field() {
        let body = br#"{"messages":[]}"#;
        let mappings = serde_json::json!({"claude-opus-4-6": "my-opus"});
        let result = transform_model(body, &mappings);
        let parsed: Value = serde_json::from_slice(&result).unwrap();
        assert!(parsed.get("model").is_none());
    }

    #[test]
    fn test_transform_model_invalid_json() {
        let body = b"not json";
        let mappings = serde_json::json!({"claude-opus-4-6": "my-opus"});
        let result = transform_model(body, &mappings);
        assert_eq!(result, body);
    }

    #[test]
    fn test_failover_status_code_matching() {
        let codes = vec![429u16, 500, 502, 503];
        assert!(codes.contains(&429));
        assert!(codes.contains(&500));
        assert!(!codes.contains(&400));
        assert!(!codes.contains(&200));
    }
}
