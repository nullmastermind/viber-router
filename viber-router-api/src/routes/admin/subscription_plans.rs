use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
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
}

async fn create_plan(
    State(state): State<AppState>,
    Json(input): Json<CreateSubscriptionPlan>,
) -> Result<(StatusCode, Json<SubscriptionPlan>), ApiError> {
    if input.sub_type != "fixed" && input.sub_type != "hourly_reset" {
        return Err(err(StatusCode::BAD_REQUEST, "sub_type must be 'fixed' or 'hourly_reset'"));
    }
    if input.sub_type == "hourly_reset" && input.reset_hours.is_none() {
        return Err(err(StatusCode::BAD_REQUEST, "reset_hours is required for hourly_reset plans"));
    }

    let model_limits = input.model_limits.unwrap_or(serde_json::json!({}));

    let plan = sqlx::query_as::<_, SubscriptionPlan>(
        "INSERT INTO subscription_plans (name, sub_type, cost_limit_usd, model_limits, reset_hours, duration_days) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
    )
    .bind(&input.name)
    .bind(&input.sub_type)
    .bind(input.cost_limit_usd)
    .bind(&model_limits)
    .bind(input.reset_hours)
    .bind(input.duration_days)
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
        && st != "fixed" && st != "hourly_reset"
    {
        return Err(err(StatusCode::BAD_REQUEST, "sub_type must be 'fixed' or 'hourly_reset'"));
    }

    let plan = sqlx::query_as::<_, SubscriptionPlan>(
        "UPDATE subscription_plans SET \
         name = COALESCE($1, name), \
         sub_type = COALESCE($2, sub_type), \
         cost_limit_usd = COALESCE($3, cost_limit_usd), \
         model_limits = COALESCE($4, model_limits), \
         reset_hours = CASE WHEN $5 THEN $6 ELSE reset_hours END, \
         duration_days = COALESCE($7, duration_days), \
         is_active = COALESCE($8, is_active), \
         updated_at = now() \
         WHERE id = $9 RETURNING *",
    )
    .bind(&input.name)
    .bind(&input.sub_type)
    .bind(input.cost_limit_usd)
    .bind(&input.model_limits)
    .bind(input.reset_hours.is_some())
    .bind(input.reset_hours.flatten())
    .bind(input.duration_days)
    .bind(input.is_active)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Plan not found"))?;

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
