use axum::{
    Router,
    body::Body,
    extract::{OriginalUri, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use futures_util::StreamExt;
use serde_json::Value;

use crate::cache;
use crate::log_buffer::{FailoverAttempt, ProxyLogEntry};
use crate::models::{CountTokensServer, GroupConfig, GroupServerDetail};
use crate::routes::AppState;
use crate::routes::key_parser::parse_api_key;
use crate::ttft_buffer::TtftLogEntry;

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
        "SELECT gs.server_id, s.short_id, s.name as server_name, s.base_url, s.api_key, gs.priority, gs.model_mappings \
         FROM group_servers gs JOIN servers s ON s.id = gs.server_id \
         WHERE gs.group_id = $1 ORDER BY gs.priority",
    )
    .bind(group.id)
    .fetch_all(&state.db)
    .await
    .ok()?;

    let failover_codes: Vec<u16> = serde_json::from_value(group.failover_status_codes.clone())
        .unwrap_or_else(|_| vec![429, 500, 502, 503]);

    // Resolve count-tokens default server if configured
    let count_tokens_server = if let Some(ct_server_id) = group.count_tokens_server_id {
        sqlx::query_as::<_, (uuid::Uuid, i32, String, String, Option<String>)>(
            "SELECT id, short_id, name, base_url, api_key FROM servers WHERE id = $1",
        )
        .bind(ct_server_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|(server_id, short_id, server_name, base_url, api_key)| CountTokensServer {
            server_id,
            short_id,
            server_name,
            base_url,
            api_key,
            model_mappings: group.count_tokens_model_mappings.clone(),
        })
    } else {
        None
    };

    let config = GroupConfig {
        group_id: group.id,
        api_key: group.api_key.clone(),
        is_active: group.is_active,
        failover_status_codes: failover_codes,
        ttft_timeout_ms: group.ttft_timeout_ms,
        servers,
        count_tokens_server,
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

/// Extract the "model" field from the request body JSON (before any mapping).
fn extract_request_model(body: &[u8]) -> Option<String> {
    serde_json::from_slice::<Value>(body)
        .ok()
        .and_then(|v| v.get("model")?.as_str().map(String::from))
}

async fn proxy_handler(
    State(state): State<AppState>,
    OriginalUri(original_uri): OriginalUri,
    req: Request,
) -> Response {
    // Extract API key from x-api-key header, falling back to Authorization: Bearer <key>
    let raw_key = match req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            req.headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        }) {
        Some(key) => key,
        None => {
            return anthropic_error(
                StatusCode::UNAUTHORIZED,
                "authentication_error",
                "Invalid API key",
            );
        }
    };

    let parsed = parse_api_key(&raw_key);

    // Look up group config using the extracted group key
    let config = match resolve_group_config(&state, &parsed.group_key).await {
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
    let mut any_server_attempted = false;

    // Extract request model before any transformation
    let request_model = extract_request_model(&body_bytes);
    let request_path = original_uri.path().to_string();
    let request_method = method.to_string();
    let loop_start = std::time::Instant::now();

    // Build headers map (excluding host, content-length, x-api-key) for logging
    let log_headers: serde_json::Map<String, Value> = headers
        .iter()
        .filter(|(name, _)| {
            *name != "host" && *name != "content-length" && *name != "x-api-key"
        })
        .filter_map(|(name, value)| {
            value.to_str().ok().map(|v| (name.to_string(), Value::String(v.to_string())))
        })
        .collect();

    // Failover chain tracking
    let mut failover_chain: Vec<FailoverAttempt> = Vec::new();
    let mut last_server_id = uuid::Uuid::nil();
    let mut last_server_name = String::new();

    // Count-tokens default server: try before the failover waterfall
    let is_count_tokens = request_path == "/v1/messages/count_tokens";
    let mut ct_default_attempted = false;

    if is_count_tokens
        && let Some(ref ct_server) = config.count_tokens_server
    {
        // Key resolution for default server: dynamic key > server default > skip
        let ct_resolved_key = if let Some(dk) = parsed.dynamic_keys.get(&ct_server.short_id) {
            Some(dk.clone())
        } else {
            ct_server.api_key.clone()
        };

        if let Some(resolved_key) = ct_resolved_key {
            ct_default_attempted = true;
            any_server_attempted = true;
            let transformed_body = transform_model(&body_bytes, &ct_server.model_mappings);

            let path = original_uri.path();
            let upstream_url = if let Some(query) = original_uri.query() {
                format!("{}{path}?{query}", ct_server.base_url.trim_end_matches('/'))
            } else {
                format!("{}{path}", ct_server.base_url.trim_end_matches('/'))
            };

            let mut upstream_req = client.request(method.clone(), &upstream_url);

            let mut server_log_headers = log_headers.clone();
            server_log_headers.insert("x-api-key".to_string(), Value::String(resolved_key.clone()));
            server_log_headers.insert("authorization".to_string(), Value::String(format!("Bearer {}", resolved_key)));

            for (name, value) in headers.iter() {
                if name == "x-api-key" || name == "authorization" || name == "host" || name == "content-length" {
                    continue;
                }
                if let Ok(reqwest_name) = reqwest::header::HeaderName::from_bytes(name.as_str().as_bytes())
                    && let Ok(reqwest_value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes())
                {
                    upstream_req = upstream_req.header(reqwest_name, reqwest_value);
                }
            }
            upstream_req = upstream_req.header("x-api-key", &resolved_key);
            upstream_req = upstream_req.header("authorization", format!("Bearer {}", resolved_key));

            let attempt_body: Option<serde_json::Value> = serde_json::from_slice(&transformed_body).ok();
            let attempt_headers = Value::Object(server_log_headers);
            let attempt_url = upstream_url.clone();

            upstream_req = upstream_req.body(transformed_body);

            let server_start = std::time::Instant::now();
            match upstream_req.send().await {
                Ok(resp) => {
                    let server_latency = server_start.elapsed().as_millis() as i32;
                    let status = resp.status().as_u16();

                    failover_chain.push(FailoverAttempt {
                        server_id: ct_server.server_id,
                        server_name: ct_server.server_name.clone(),
                        status,
                        latency_ms: server_latency,
                        resolved_key: Some(resolved_key.clone()),
                        upstream_url: Some(attempt_url),
                        request_headers: Some(attempt_headers),
                        request_body: attempt_body,
                    });
                    last_server_id = ct_server.server_id;
                    last_server_name = ct_server.server_name.clone();

                    if status == 200 {
                        if failover_chain.len() > 1 {
                            emit_log_entry(
                                &state, &config, &parsed.group_key,
                                last_server_id, &last_server_name,
                                &request_path, &request_method,
                                status as i16, "failover_success",
                                loop_start.elapsed().as_millis() as i32,
                                &failover_chain, &request_model,
                                None, None, None,
                            );
                        }
                        return build_response(resp).await;
                    } else if !config.failover_status_codes.contains(&status) {
                        emit_log_entry(
                            &state, &config, &parsed.group_key,
                            last_server_id, &last_server_name,
                            &request_path, &request_method,
                            status as i16, "upstream_error",
                            loop_start.elapsed().as_millis() as i32,
                            &failover_chain, &request_model,
                            None, None, None,
                        );
                        return build_response(resp).await;
                    }
                    // Failover status code — fall through to waterfall
                }
                Err(_) => {
                    failover_chain.push(FailoverAttempt {
                        server_id: ct_server.server_id,
                        server_name: ct_server.server_name.clone(),
                        status: 0,
                        latency_ms: server_start.elapsed().as_millis() as i32,
                        resolved_key: Some(resolved_key.clone()),
                        upstream_url: Some(attempt_url),
                        request_headers: Some(attempt_headers),
                        request_body: attempt_body,
                    });
                    last_server_id = ct_server.server_id;
                    last_server_name = ct_server.server_name.clone();
                }
            }
        }
    }

    // Failover waterfall with key resolution
    for (server_idx, server) in config.servers.iter().enumerate() {
        // Skip the count-tokens default server if already attempted
        if ct_default_attempted
            && let Some(ref ct) = config.count_tokens_server
            && server.server_id == ct.server_id
        {
            continue;
        }

        // Key resolution: dynamic key > server default > skip
        let resolved_key = if let Some(dk) = parsed.dynamic_keys.get(&server.short_id) {
            dk.clone()
        } else if let Some(ref default_key) = server.api_key {
            default_key.clone()
        } else {
            continue; // No key available — skip this server
        };

        any_server_attempted = true;
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

        // Build per-server log headers (with resolved key)
        let mut server_log_headers = log_headers.clone();
        server_log_headers.insert("x-api-key".to_string(), Value::String(resolved_key.clone()));
        server_log_headers.insert("authorization".to_string(), Value::String(format!("Bearer {}", resolved_key)));

        // Forward headers, replacing x-api-key and authorization with resolved key
        for (name, value) in headers.iter() {
            if name == "x-api-key" || name == "authorization" {
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
        upstream_req = upstream_req.header("x-api-key", &resolved_key);
        upstream_req = upstream_req.header("authorization", format!("Bearer {}", resolved_key));

        // Prepare curl data for this attempt
        let attempt_body: Option<serde_json::Value> = serde_json::from_slice(&transformed_body).ok();
        let attempt_headers = Value::Object(server_log_headers);
        let attempt_url = upstream_url.clone();

        upstream_req = upstream_req.body(transformed_body);

        let server_start = std::time::Instant::now();
        let upstream_resp = match upstream_req.send().await {
            Ok(resp) => resp,
            Err(_) => {
                // Connection error → record attempt and try next server
                failover_chain.push(FailoverAttempt {
                    server_id: server.server_id,
                    server_name: server.server_name.clone(),
                    status: 0,
                    latency_ms: server_start.elapsed().as_millis() as i32,
                    resolved_key: Some(resolved_key.clone()),
                    upstream_url: Some(attempt_url),
                    request_headers: Some(attempt_headers),
                    request_body: attempt_body,
                });
                last_server_id = server.server_id;
                last_server_name = server.server_name.clone();
                continue;
            }
        };

        let server_latency = server_start.elapsed().as_millis() as i32;
        let status = upstream_resp.status().as_u16();

        failover_chain.push(FailoverAttempt {
            server_id: server.server_id,
            server_name: server.server_name.clone(),
            status,
            latency_ms: server_latency,
            resolved_key: Some(resolved_key.clone()),
            upstream_url: Some(attempt_url),
            request_headers: Some(attempt_headers),
            request_body: attempt_body,
        });
        last_server_id = server.server_id;
        last_server_name = server.server_name.clone();

        // Check if this is a failover status code
        if config.failover_status_codes.contains(&status) {
            continue;
        }

        // Non-failover error — log it
        if status != 200 {
            emit_log_entry(
                &state, &config, &parsed.group_key,
                last_server_id, &last_server_name,
                &request_path, &request_method,
                status as i16, "upstream_error",
                loop_start.elapsed().as_millis() as i32,
                &failover_chain, &request_model,
                None, None, None,
            );
            return build_response(upstream_resp).await;
        }

        // Check if this is an SSE response
        let is_sse = upstream_resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|ct| ct.contains("text/event-stream"));

        if !is_sse {
            // Non-SSE: log failover chain if applicable, then use existing path
            if failover_chain.len() > 1 {
                emit_log_entry(
                    &state, &config, &parsed.group_key,
                    last_server_id, &last_server_name,
                    &request_path, &request_method,
                    status as i16, "failover_success",
                    loop_start.elapsed().as_millis() as i32,
                    &failover_chain, &request_model,
                    None, None, None,
                );
            }
            return build_response(upstream_resp).await;
        }

        // SSE response — measure TTFT
        let ttft_enabled = config.ttft_timeout_ms.is_some();
        let total_servers = config.servers.len();
        let is_last_server = server_idx == total_servers - 1;
        let should_timeout = ttft_enabled && total_servers > 1 && !is_last_server;

        // Build response headers before consuming the stream
        let resp_status = StatusCode::from_u16(upstream_resp.status().as_u16())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response_headers = HeaderMap::new();
        for (name, value) in upstream_resp.headers().iter() {
            if let Ok(axum_name) = axum::http::header::HeaderName::from_bytes(name.as_str().as_bytes())
                && let Ok(axum_value) = HeaderValue::from_bytes(value.as_bytes())
            {
                response_headers.insert(axum_name, axum_value);
            }
        }

        let mut stream = upstream_resp.bytes_stream();

        if should_timeout {
            let timeout_ms = config.ttft_timeout_ms.unwrap() as u64;
            let elapsed_ms = server_start.elapsed().as_millis() as u64;

            if elapsed_ms >= timeout_ms {
                // Already exceeded TTFT threshold waiting for headers — failover immediately
                drop(stream);
                if let Some(last) = failover_chain.last_mut() {
                    last.status = 0;
                }
                emit_ttft_entry(&state, config.group_id, server.server_id, &request_model, None, true, &request_path);
                continue;
            }

            let remaining_ms = timeout_ms - elapsed_ms;
            match tokio::time::timeout(
                std::time::Duration::from_millis(remaining_ms),
                stream.next(),
            ).await {
                Ok(Some(Ok(first_chunk))) => {
                    // First chunk received within timeout
                    let ttft_ms = server_start.elapsed().as_millis() as i32;
                    emit_ttft_entry(&state, config.group_id, server.server_id, &request_model, Some(ttft_ms), false, &request_path);

                    // Log failover chain if this wasn't the first server tried
                    if failover_chain.len() > 1 {
                        emit_log_entry(
                            &state, &config, &parsed.group_key,
                            last_server_id, &last_server_name,
                            &request_path, &request_method,
                            status as i16, "failover_success",
                            loop_start.elapsed().as_millis() as i32,
                            &failover_chain, &request_model,
                            None, None, None,
                        );
                    }

                    let first = futures_util::stream::once(async move { Ok::<_, std::io::Error>(first_chunk) });
                    let rest = stream.map(|chunk| chunk.map_err(std::io::Error::other));
                    let body = Body::from_stream(first.chain(rest));
                    let mut resp = Response::builder().status(resp_status);
                    *resp.headers_mut().unwrap() = response_headers;
                    return resp.body(body).unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
                }
                Ok(Some(Err(_))) | Ok(None) => {
                    // Empty stream or stream error — treat as connection error, try next
                    if let Some(last) = failover_chain.last_mut() {
                        last.status = 0;
                    }
                    emit_ttft_entry(&state, config.group_id, server.server_id, &request_model, None, false, &request_path);
                    continue;
                }
                Err(_) => {
                    // Timeout — record timed_out, drop stream, try next server
                    drop(stream);
                    if let Some(last) = failover_chain.last_mut() {
                        last.status = 0;
                    }
                    emit_ttft_entry(&state, config.group_id, server.server_id, &request_model, None, true, &request_path);
                    continue;
                }
            }
        } else {
            // No timeout: measure TTFT but always wait
            match stream.next().await {
                Some(Ok(first_chunk)) => {
                    let ttft_ms = server_start.elapsed().as_millis() as i32;
                    emit_ttft_entry(&state, config.group_id, server.server_id, &request_model, Some(ttft_ms), false, &request_path);

                    // Log failover chain if this wasn't the first server tried
                    if failover_chain.len() > 1 {
                        emit_log_entry(
                            &state, &config, &parsed.group_key,
                            last_server_id, &last_server_name,
                            &request_path, &request_method,
                            status as i16, "failover_success",
                            loop_start.elapsed().as_millis() as i32,
                            &failover_chain, &request_model,
                            None, None, None,
                        );
                    }

                    let first = futures_util::stream::once(async move { Ok::<_, std::io::Error>(first_chunk) });
                    let rest = stream.map(|chunk| chunk.map_err(std::io::Error::other));
                    let body = Body::from_stream(first.chain(rest));
                    let mut resp = Response::builder().status(resp_status);
                    *resp.headers_mut().unwrap() = response_headers;
                    return resp.body(body).unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
                }
                Some(Err(_)) | None => {
                    // Empty stream or error — treat as connection error
                    emit_ttft_entry(&state, config.group_id, server.server_id, &request_model, None, false, &request_path);
                    continue;
                }
            }
        }
    }

    // All servers skipped — no key available for any server
    if !any_server_attempted {
        return anthropic_error(
            StatusCode::UNAUTHORIZED,
            "authentication_error",
            "No server keys configured",
        );
    }

    // All servers with keys exhausted (failover codes or connection errors)
    let all_connection_errors = failover_chain.iter().all(|a| a.status == 0);
    let error_type = if all_connection_errors {
        "connection_error"
    } else {
        "all_servers_exhausted"
    };
    // Use the last non-zero status from the chain, or 429 if all were connection errors
    let final_status: i16 = failover_chain
        .iter()
        .rev()
        .find(|a| a.status != 0)
        .map(|a| a.status as i16)
        .unwrap_or(429);

    emit_log_entry(
        &state, &config, &parsed.group_key,
        last_server_id, &last_server_name,
        &request_path, &request_method,
        final_status, error_type,
        loop_start.elapsed().as_millis() as i32,
        &failover_chain, &request_model,
        None, None, None,
    );

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

#[allow(clippy::too_many_arguments)]
fn emit_log_entry(
    state: &AppState,
    config: &GroupConfig,
    group_api_key: &str,
    server_id: uuid::Uuid,
    server_name: &str,
    request_path: &str,
    request_method: &str,
    status_code: i16,
    error_type: &str,
    latency_ms: i32,
    failover_chain: &[FailoverAttempt],
    request_model: &Option<String>,
    request_body: Option<serde_json::Value>,
    request_headers: Option<serde_json::Value>,
    upstream_url: Option<String>,
) {
    let entry = ProxyLogEntry {
        group_id: config.group_id,
        group_api_key: group_api_key.to_string(),
        server_id,
        server_name: server_name.to_string(),
        request_path: request_path.to_string(),
        request_method: request_method.to_string(),
        status_code,
        error_type: error_type.to_string(),
        latency_ms,
        failover_chain: failover_chain.to_vec(),
        request_model: request_model.clone(),
        request_body,
        request_headers,
        upstream_url,
        created_at: Utc::now(),
    };

    if state.log_tx.try_send(entry).is_err() {
        tracing::warn!("Log buffer full, dropping proxy log entry");
    }
}

fn emit_ttft_entry(
    state: &AppState,
    group_id: uuid::Uuid,
    server_id: uuid::Uuid,
    request_model: &Option<String>,
    ttft_ms: Option<i32>,
    timed_out: bool,
    request_path: &str,
) {
    let entry = TtftLogEntry {
        group_id,
        server_id,
        request_model: request_model.clone(),
        ttft_ms,
        timed_out,
        request_path: request_path.to_string(),
        created_at: Utc::now(),
    };

    if state.ttft_tx.try_send(entry).is_err() {
        tracing::warn!("TTFT buffer full, dropping TTFT log entry");
    }
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
