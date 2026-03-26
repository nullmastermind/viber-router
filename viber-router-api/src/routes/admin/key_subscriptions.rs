use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use deadpool_redis::redis::AsyncCommands;
use uuid::Uuid;

use crate::models::{AssignSubscription, CancelSubscription, KeySubscription, SubscriptionPlan};
use crate::routes::AppState;

type ApiError = (StatusCode, Json<serde_json::Value>);

#[derive(serde::Serialize)]
struct KeySubscriptionWithUsage {
    #[serde(flatten)]
    sub: KeySubscription,
    cost_used: f64,
}

fn err(status: StatusCode, msg: &str) -> ApiError {
    (status, Json(serde_json::json!({"error": msg})))
}

fn internal(e: impl std::fmt::Display) -> ApiError {
    err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_subscriptions).post(assign_subscription))
        .route("/{sub_id}", axum::routing::patch(cancel_subscription))
}

async fn assign_subscription(
    State(state): State<AppState>,
    Path((_group_id, key_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<AssignSubscription>,
) -> Result<(StatusCode, Json<KeySubscription>), ApiError> {
    let plan = sqlx::query_as::<_, SubscriptionPlan>(
        "SELECT * FROM subscription_plans WHERE id = $1",
    )
    .bind(input.plan_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Plan not found"))?;

    if !plan.is_active {
        return Err(err(StatusCode::BAD_REQUEST, "Plan is not active"));
    }

    let sub = sqlx::query_as::<_, KeySubscription>(
        "INSERT INTO key_subscriptions (group_key_id, plan_id, sub_type, cost_limit_usd, model_limits, reset_hours, duration_days) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
    )
    .bind(key_id)
    .bind(plan.id)
    .bind(&plan.sub_type)
    .bind(plan.cost_limit_usd)
    .bind(&plan.model_limits)
    .bind(plan.reset_hours)
    .bind(plan.duration_days)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    // Invalidate subscription list cache
    if let Ok(mut conn) = state.redis.get().await {
        let _: Result<(), _> = conn.del(format!("key_subs:{key_id}")).await;
    }

    Ok((StatusCode::CREATED, Json(sub)))
}

async fn list_subscriptions(
    State(state): State<AppState>,
    Path((_group_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<KeySubscriptionWithUsage>>, ApiError> {
    let subs = sqlx::query_as::<_, KeySubscription>(
        "SELECT * FROM key_subscriptions WHERE group_key_id = $1 ORDER BY created_at DESC",
    )
    .bind(key_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    let mut result = Vec::with_capacity(subs.len());
    for sub in subs {
        let cost_used = crate::subscription::get_total_cost(&state, &sub).await;
        result.push(KeySubscriptionWithUsage { sub, cost_used });
    }

    Ok(Json(result))
}

async fn cancel_subscription(
    State(state): State<AppState>,
    Path((_group_id, key_id, sub_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(input): Json<CancelSubscription>,
) -> Result<Json<KeySubscription>, ApiError> {
    if input.status != "cancelled" {
        return Err(err(StatusCode::BAD_REQUEST, "Only 'cancelled' status is supported"));
    }

    let current = sqlx::query_as::<_, KeySubscription>(
        "SELECT * FROM key_subscriptions WHERE id = $1 AND group_key_id = $2",
    )
    .bind(sub_id)
    .bind(key_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Subscription not found"))?;

    if current.status != "active" {
        return Err(err(StatusCode::BAD_REQUEST, "Only active subscriptions can be cancelled"));
    }

    let sub = sqlx::query_as::<_, KeySubscription>(
        "UPDATE key_subscriptions SET status = 'cancelled' WHERE id = $1 RETURNING *",
    )
    .bind(sub_id)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    // Invalidate subscription list cache
    if let Ok(mut conn) = state.redis.get().await {
        let _: Result<(), _> = conn.del(format!("key_subs:{key_id}")).await;
    }

    Ok(Json(sub))
}
