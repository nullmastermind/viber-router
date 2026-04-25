use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use deadpool_redis::redis::AsyncCommands;
use serde::Deserialize;
use uuid::Uuid;

use crate::models::{AssignSubscription, CancelSubscription, KeySubscription, PaginatedResponse, SubscriptionPlan};
use crate::routes::AppState;

type ApiError = (StatusCode, Json<serde_json::Value>);

#[derive(serde::Serialize)]
struct KeySubscriptionWithUsage {
    #[serde(flatten)]
    sub: KeySubscription,
    cost_used: f64,
}

#[derive(Debug, Deserialize)]
struct ListParams {
    page: Option<i64>,
    limit: Option<i64>,
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
    // Detect bonus creation path: has bonus fields, no plan_id
    if input.bonus_name.is_some() || input.bonus_base_url.is_some() || input.bonus_api_key.is_some() {
        // Validate required bonus fields
        let bonus_name = input.bonus_name.as_deref().filter(|s| !s.is_empty())
            .ok_or_else(|| err(StatusCode::BAD_REQUEST, "bonus_name is required"))?;
        let bonus_base_url = input.bonus_base_url.as_deref().filter(|s| !s.is_empty())
            .ok_or_else(|| err(StatusCode::BAD_REQUEST, "bonus_base_url is required"))?;
        let bonus_api_key = input.bonus_api_key.as_deref().filter(|s| !s.is_empty())
            .ok_or_else(|| err(StatusCode::BAD_REQUEST, "bonus_api_key is required"))?;

        let sub = sqlx::query_as::<_, KeySubscription>(
            "INSERT INTO key_subscriptions \
             (group_key_id, plan_id, sub_type, cost_limit_usd, model_limits, model_request_costs, \
              reset_hours, duration_days, rpm_limit, bonus_name, bonus_base_url, bonus_api_key, \
              bonus_quota_url, bonus_quota_headers) \
             VALUES ($1, NULL, 'bonus', 0, '{}', '{}', NULL, 36500, NULL, $2, $3, $4, $5, $6) \
             RETURNING *",
        )
        .bind(key_id)
        .bind(bonus_name)
        .bind(bonus_base_url)
        .bind(bonus_api_key)
        .bind(&input.bonus_quota_url)
        .bind(&input.bonus_quota_headers)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

        // Invalidate subscription list cache
        if let Ok(mut conn) = state.redis.get().await {
            let _: Result<(), _> = conn.del(format!("key_subs:{key_id}")).await;
        }

        return Ok((StatusCode::CREATED, Json(sub)));
    }

    // Standard plan-based creation
    let plan_id = input.plan_id
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "Either plan_id or bonus fields (bonus_name, bonus_base_url, bonus_api_key) are required"))?;

    let plan = sqlx::query_as::<_, SubscriptionPlan>(
        "SELECT * FROM subscription_plans WHERE id = $1",
    )
    .bind(plan_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Plan not found"))?;

    if !plan.is_active {
        return Err(err(StatusCode::BAD_REQUEST, "Plan is not active"));
    }

    let sub = sqlx::query_as::<_, KeySubscription>(
        "INSERT INTO key_subscriptions (group_key_id, plan_id, sub_type, cost_limit_usd, model_limits, model_request_costs, reset_hours, duration_days, rpm_limit) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *",
    )
    .bind(key_id)
    .bind(plan.id)
    .bind(&plan.sub_type)
    .bind(plan.cost_limit_usd)
    .bind(&plan.model_limits)
    .bind(&plan.model_request_costs)
    .bind(plan.reset_hours)
    .bind(plan.duration_days)
    .bind(plan.rpm_limit)
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
    Query(params): Query<ListParams>,
) -> Result<Json<PaginatedResponse<KeySubscriptionWithUsage>>, ApiError> {
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * limit;

    let (total,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM key_subscriptions WHERE group_key_id = $1",
    )
    .bind(key_id)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    let subs = sqlx::query_as::<_, KeySubscription>(
        "SELECT * FROM key_subscriptions WHERE group_key_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(key_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    let mut data = Vec::with_capacity(subs.len());
    for sub in subs {
        let cost_used = crate::subscription::get_total_cost(&state, &sub).await;
        data.push(KeySubscriptionWithUsage { sub, cost_used });
    }

    let total_pages = (total as f64 / limit as f64).ceil() as i64;
    Ok(Json(PaginatedResponse { data, total, page, total_pages }))
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
