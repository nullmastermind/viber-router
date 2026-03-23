use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};

use super::AppState;

pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();

    let redis_ok = async {
        let mut conn = state.redis.get().await?;
        deadpool_redis::redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await?;
        Ok::<_, anyhow::Error>(())
    }
    .await
    .is_ok();

    let status = if db_ok && redis_ok { "ok" } else { "error" };
    let db_status = if db_ok { "ok" } else { "error" };
    let redis_status = if redis_ok { "ok" } else { "error" };

    let code = if db_ok && redis_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (code, Json(json!({ "status": status, "db": db_status, "redis": redis_status })))
}
