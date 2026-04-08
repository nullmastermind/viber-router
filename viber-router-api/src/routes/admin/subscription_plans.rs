use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use deadpool_redis::redis::AsyncCommands;
use uuid::Uuid;

use crate::models::{CreateSubscriptionPlan, SubscriptionPlan, UpdateSubscriptionPlan};
use crate::routes::AppState;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn err(status: StatusCode, msg: &str) -> ApiError {
    (status, Json(serde_json::json!({"error": msg})))
}

fn internal(e: impl std::fmt::Display) -> ApiError {
    err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_plans).post(create_plan))
        .route("/{id}", get(get_plan).patch(update_plan).delete(delete_plan))
        .route("/{id}/sync-rpm", axum::routing::post(sync_rpm))
}

async fn create_plan(
    State(state): State<AppState>,
    Json(input): Json<CreateSubscriptionPlan>,
) -> Result<(StatusCode, Json<SubscriptionPlan>), ApiError> {
    if input.sub_type != "fixed" && input.sub_type != "hourly_reset" && input.sub_type != "pay_per_request" {
        return Err(err(StatusCode::BAD_REQUEST, "sub_type must be 'fixed', 'hourly_reset', or 'pay_per_request'"));
    }
    if input.sub_type == "hourly_reset" && input.reset_hours.is_none() {
        return Err(err(StatusCode::BAD_REQUEST, "reset_hours is required for hourly_reset plans"));
    }

    let model_limits = input.model_limits.unwrap_or(serde_json::json!({}));
    let model_request_costs = input.model_request_costs.unwrap_or(serde_json::json!({}));

    if input.sub_type == "pay_per_request"
        && model_request_costs.as_object().map(|o| o.is_empty()).unwrap_or(true)
    {
        return Err(err(StatusCode::BAD_REQUEST, "model_request_costs must not be empty for pay_per_request plans"));
    }

    let plan = sqlx::query_as::<_, SubscriptionPlan>(
        "INSERT INTO subscription_plans (name, sub_type, cost_limit_usd, model_limits, model_request_costs, reset_hours, duration_days, rpm_limit) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
    )
    .bind(&input.name)
    .bind(&input.sub_type)
    .bind(input.cost_limit_usd)
    .bind(&model_limits)
    .bind(&model_request_costs)
    .bind(input.reset_hours)
    .bind(input.duration_days)
    .bind(input.rpm_limit)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    Ok((StatusCode::CREATED, Json(plan)))
}

async fn list_plans(
    State(state): State<AppState>,
) -> Result<Json<Vec<SubscriptionPlan>>, ApiError> {
    let plans = sqlx::query_as::<_, SubscriptionPlan>(
        "SELECT * FROM subscription_plans ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    Ok(Json(plans))
}

async fn get_plan(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SubscriptionPlan>, ApiError> {
    let plan = sqlx::query_as::<_, SubscriptionPlan>(
        "SELECT * FROM subscription_plans WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Plan not found"))?;

    Ok(Json(plan))
}

async fn update_plan(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateSubscriptionPlan>,
) -> Result<Json<SubscriptionPlan>, ApiError> {
    if let Some(ref st) = input.sub_type
        && st != "fixed" && st != "hourly_reset" && st != "pay_per_request"
    {
        return Err(err(StatusCode::BAD_REQUEST, "sub_type must be 'fixed', 'hourly_reset', or 'pay_per_request'"));
    }

    let plan = sqlx::query_as::<_, SubscriptionPlan>(
        "UPDATE subscription_plans SET \
         name = COALESCE($1, name), \
         sub_type = COALESCE($2, sub_type), \
         cost_limit_usd = COALESCE($3, cost_limit_usd), \
         model_limits = COALESCE($4, model_limits), \
         model_request_costs = COALESCE($5, model_request_costs), \
         reset_hours = CASE WHEN $6 THEN $7 ELSE reset_hours END, \
         duration_days = COALESCE($8, duration_days), \
         is_active = COALESCE($9, is_active), \
         rpm_limit = CASE WHEN $10 THEN $11 ELSE rpm_limit END, \
         updated_at = now() \
         WHERE id = $12 RETURNING *",
    )
    .bind(&input.name)
    .bind(&input.sub_type)
    .bind(input.cost_limit_usd)
    .bind(&input.model_limits)
    .bind(&input.model_request_costs)
    .bind(input.reset_hours.is_some())
    .bind(input.reset_hours.flatten())
    .bind(input.duration_days)
    .bind(input.is_active)
    .bind(input.rpm_limit.is_some())
    .bind(input.rpm_limit.flatten())
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Plan not found"))?;

    // Auto-sync rpm_limit to active subscriptions when rpm_limit is updated
    if input.rpm_limit.is_some() {
        let _ = sqlx::query(
            "UPDATE key_subscriptions SET rpm_limit = $1 WHERE plan_id = $2 AND status = 'active'",
        )
        .bind(plan.rpm_limit)
        .bind(id)
        .execute(&state.db)
        .await;

        let key_ids: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT DISTINCT group_key_id FROM key_subscriptions WHERE plan_id = $1 AND status = 'active'",
        )
        .bind(id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        if let Ok(mut conn) = state.redis.get().await {
            for (key_id,) in &key_ids {
                let _: Result<(), _> = conn.del(format!("key_subs:{key_id}")).await;
            }
        }
    }

    Ok(Json(plan))
}

async fn delete_plan(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let has_subs = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM key_subscriptions WHERE plan_id = $1)",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    if has_subs {
        return Err(err(StatusCode::CONFLICT, "Plan has existing subscriptions"));
    }

    let result = sqlx::query("DELETE FROM subscription_plans WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal)?;

    if result.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Plan not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Serialize)]
struct SyncResult {
    updated: u64,
}

async fn sync_rpm(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SyncResult>, ApiError> {
    let plan = sqlx::query_as::<_, SubscriptionPlan>(
        "SELECT * FROM subscription_plans WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Plan not found"))?;

    // Update rpm_limit on all active subscriptions from this plan
    let result = sqlx::query(
        "UPDATE key_subscriptions SET rpm_limit = $1 WHERE plan_id = $2 AND status = 'active'",
    )
    .bind(plan.rpm_limit)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(internal)?;

    // Invalidate subscription list caches for affected keys
    let key_ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT DISTINCT group_key_id FROM key_subscriptions WHERE plan_id = $1 AND status = 'active'",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if let Ok(mut conn) = state.redis.get().await {
        for (key_id,) in &key_ids {
            let _: Result<(), _> = conn.del(format!("key_subs:{key_id}")).await;
        }
    }

    Ok(Json(SyncResult { updated: result.rows_affected() }))
}
