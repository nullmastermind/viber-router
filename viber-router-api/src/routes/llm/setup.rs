use axum::extract::Query;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

const CLAUDECODE_LINUX: &str = include_str!("../../../templates/setup/claudecode-linux.sh.tpl");
const CLAUDECODE_WINDOWS: &str =
    include_str!("../../../templates/setup/claudecode-windows.ps1.tpl");
const CODEX_LINUX: &str = include_str!("../../../templates/setup/codex-linux.sh.tpl");
const CODEX_WINDOWS: &str = include_str!("../../../templates/setup/codex-windows.ps1.tpl");

#[derive(Debug, Deserialize)]
pub struct ClaudeCodeQuery {
    pub key: String,
    #[serde(default)]
    pub os: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub haiku: Option<String>,
    #[serde(default)]
    pub sonnet: Option<String>,
    #[serde(default)]
    pub opus: Option<String>,
    #[serde(default)]
    pub subagent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CodexQuery {
    pub key: String,
    #[serde(default)]
    pub os: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub small: Option<String>,
    #[serde(default)]
    pub medium: Option<String>,
    #[serde(default)]
    pub large: Option<String>,
}

fn is_windows(os: Option<&str>) -> bool {
    matches!(os, Some(s) if s.eq_ignore_ascii_case("windows") || s.eq_ignore_ascii_case("win"))
}

fn derive_endpoint(headers: &HeaderMap, override_val: Option<&str>) -> String {
    if let Some(v) = override_val
        && !v.is_empty()
    {
        return v.trim_end_matches('/').to_string();
    }
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("http");
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost");
    format!("{scheme}://{host}")
}

fn validate_model(value: Option<String>, default: &str) -> String {
    let v = value.unwrap_or_default();
    let cleaned: String = v
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
        .collect();
    if cleaned.is_empty() {
        default.to_string()
    } else {
        cleaned
    }
}

fn validate_key(key: &str) -> Result<&str, StatusCode> {
    if key.is_empty() || key.len() > 200 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if !key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(key)
}

fn render(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("{{{{{}}}}}", k), v);
    }
    out
}

fn script_response(body: String, windows: bool) -> Response {
    let mut resp = body.into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(if windows {
            "text/plain; charset=utf-8"
        } else {
            "text/x-shellscript; charset=utf-8"
        }),
    );
    resp
}

pub async fn setup_claudecode(
    headers: HeaderMap,
    Query(q): Query<ClaudeCodeQuery>,
) -> Result<Response, StatusCode> {
    let key = validate_key(&q.key)?;
    let endpoint = derive_endpoint(&headers, q.endpoint.as_deref());
    let haiku = validate_model(q.haiku, "claude-haiku-4-5");
    let sonnet = validate_model(q.sonnet, "claude-sonnet-4-6");
    let opus = validate_model(q.opus, "claude-opus-4-6");
    let subagent = validate_model(q.subagent, &sonnet);
    let windows = is_windows(q.os.as_deref());
    let template = if windows {
        CLAUDECODE_WINDOWS
    } else {
        CLAUDECODE_LINUX
    };
    let body = render(
        template,
        &[
            ("API_KEY", key),
            ("ENDPOINT_URL", &endpoint),
            ("HAIKU", &haiku),
            ("SONNET", &sonnet),
            ("OPUS", &opus),
            ("SUBAGENT", &subagent),
        ],
    );
    Ok(script_response(body, windows))
}

pub async fn setup_codex(
    headers: HeaderMap,
    Query(q): Query<CodexQuery>,
) -> Result<Response, StatusCode> {
    let key = validate_key(&q.key)?;
    let endpoint = derive_endpoint(&headers, q.endpoint.as_deref());
    let small = validate_model(q.small, "claude-haiku-4-5");
    let medium = validate_model(q.medium, "claude-sonnet-4-6");
    let large = validate_model(q.large, "claude-opus-4-6");
    let windows = is_windows(q.os.as_deref());
    let template = if windows { CODEX_WINDOWS } else { CODEX_LINUX };
    let body = render(
        template,
        &[
            ("API_KEY", key),
            ("ENDPOINT_URL", &endpoint),
            ("SMALL", &small),
            ("MEDIUM", &medium),
            ("LARGE", &large),
        ],
    );
    Ok(script_response(body, windows))
}
