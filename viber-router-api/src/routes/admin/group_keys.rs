use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::models::{
    CreateGroupKey, GroupKey, PaginatedResponse, SubscriptionPlan, UpdateGroupKey, generate_api_key,
};
use crate::routes::AppState;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn err(status: StatusCode, msg: &str) -> ApiError {
    (status, Json(serde_json::json!({"error": msg})))
}

fn internal(e: impl std::fmt::Display) -> ApiError {
    err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkCreateKeys {
    pub count: u32,
    pub plan_id: Uuid,
    pub name_prefix: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_keys).post(create_key))
        .route("/bulk", post(bulk_create_keys))
        .route("/{key_id}", patch(update_key))
        .route("/{key_id}/regenerate", post(regenerate_key))
        .nest(
            "/{key_id}/allowed-models",
            super::group_key_allowed_models::router(),
        )
        .nest(
            "/{key_id}/subscriptions",
            super::key_subscriptions::router(),
        )
        .nest("/{key_id}/servers", super::group_key_servers::router())
}

async fn create_key(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Json(input): Json<CreateGroupKey>,
) -> Result<(StatusCode, Json<GroupKey>), ApiError> {
    if input.name.len() > 100 {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "Name must be 100 characters or less",
        ));
    }

    // Verify group exists
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM groups WHERE id = $1)")
        .bind(group_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

    if !exists {
        return Err(err(StatusCode::NOT_FOUND, "Group not found"));
    }

    let api_key = generate_api_key();
    let key = sqlx::query_as::<_, GroupKey>(
        "INSERT INTO group_keys (group_id, api_key, name) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(group_id)
    .bind(&api_key)
    .bind(&input.name)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    Ok((StatusCode::CREATED, Json(key)))
}

async fn bulk_create_keys(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Json(input): Json<BulkCreateKeys>,
) -> Result<(StatusCode, Json<Vec<GroupKey>>), ApiError> {
    if input.count == 0 || input.count > 500 {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "Count must be between 1 and 500",
        ));
    }

    // Verify group exists
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM groups WHERE id = $1)")
        .bind(group_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;
    if !exists {
        return Err(err(StatusCode::NOT_FOUND, "Group not found"));
    }

    // Fetch and validate plan
    let plan =
        sqlx::query_as::<_, SubscriptionPlan>("SELECT * FROM subscription_plans WHERE id = $1")
            .bind(input.plan_id)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?
            .ok_or_else(|| err(StatusCode::NOT_FOUND, "Plan not found"))?;

    if !plan.is_active {
        return Err(err(StatusCode::BAD_REQUEST, "Plan is not active"));
    }

    let mut tx = state.db.begin().await.map_err(internal)?;
    let mut created_keys: Vec<GroupKey> = Vec::with_capacity(input.count as usize);

    for i in 1..=input.count {
        let name = match &input.name_prefix {
            Some(prefix) if !prefix.is_empty() => format!("{}-{}-{}", prefix, plan.name, i),
            _ => format!("{}-{}", plan.name, i),
        };

        if name.len() > 100 {
            return Err(err(
                StatusCode::BAD_REQUEST,
                "Generated key name exceeds 100 characters. Use a shorter prefix or plan name.",
            ));
        }

        let api_key = generate_api_key();
        let key = sqlx::query_as::<_, GroupKey>(
            "INSERT INTO group_keys (group_id, api_key, name) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(group_id)
        .bind(&api_key)
        .bind(&name)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;

        sqlx::query(
            "INSERT INTO key_subscriptions (group_key_id, plan_id, sub_type, cost_limit_usd, model_limits, model_request_costs, reset_hours, duration_days, rpm_limit, tpm_limit) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(key.id)
        .bind(plan.id)
        .bind(&plan.sub_type)
        .bind(plan.cost_limit_usd)
        .bind(&plan.model_limits)
        .bind(&plan.model_request_costs)
        .bind(plan.reset_hours)
        .bind(plan.duration_days)
        .bind(plan.rpm_limit)
        .bind(plan.tpm_limit)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

        created_keys.push(key);
    }

    tx.commit().await.map_err(internal)?;

    Ok((StatusCode::CREATED, Json(created_keys)))
}

async fn list_keys(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<Json<PaginatedResponse<GroupKey>>, ApiError> {
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let offset = (page - 1) * limit;

    let (total, keys) = if let Some(ref search) = params.search {
        let pattern = format!("%{search}%");
        let (count,) = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM group_keys WHERE group_id = $1 AND (name ILIKE $2 OR api_key ILIKE $2)",
        )
        .bind(group_id)
        .bind(&pattern)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

        let rows = sqlx::query_as::<_, GroupKey>(
            "SELECT * FROM group_keys WHERE group_id = $1 AND (name ILIKE $2 OR api_key ILIKE $2) \
             ORDER BY created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(group_id)
        .bind(&pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(internal)?;

        (count, rows)
    } else {
        let (count,) =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM group_keys WHERE group_id = $1")
                .bind(group_id)
                .fetch_one(&state.db)
                .await
                .map_err(internal)?;

        let rows = sqlx::query_as::<_, GroupKey>(
            "SELECT * FROM group_keys WHERE group_id = $1 \
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(group_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(internal)?;

        (count, rows)
    };

    let total_pages = (total as f64 / limit as f64).ceil() as i64;
    Ok(Json(PaginatedResponse {
        data: keys,
        total,
        page,
        total_pages,
    }))
}

async fn update_key(
    State(state): State<AppState>,
    Path((group_id, key_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateGroupKey>,
) -> Result<Json<GroupKey>, ApiError> {
    if let Some(ref name) = input.name
        && name.len() > 100
    {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "Name must be 100 characters or less",
        ));
    }

    let key = sqlx::query_as::<_, GroupKey>(
        "UPDATE group_keys SET \
         name = COALESCE($1, name), \
         is_active = COALESCE($2, is_active), \
         updated_at = now() \
         WHERE id = $3 AND group_id = $4 RETURNING *",
    )
    .bind(&input.name)
    .bind(input.is_active)
    .bind(key_id)
    .bind(group_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Key not found"))?;

    // Invalidate cache for this sub-key
    crate::cache::invalidate_group_config(&state.redis, &key.api_key).await;
    Ok(Json(key))
}

async fn regenerate_key(
    State(state): State<AppState>,
    Path((group_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<GroupKey>, ApiError> {
    let old_key =
        sqlx::query_as::<_, GroupKey>("SELECT * FROM group_keys WHERE id = $1 AND group_id = $2")
            .bind(key_id)
            .bind(group_id)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?
            .ok_or_else(|| err(StatusCode::NOT_FOUND, "Key not found"))?;

    let new_api_key = generate_api_key();
    let key = sqlx::query_as::<_, GroupKey>(
        "UPDATE group_keys SET api_key = $1, updated_at = now() WHERE id = $2 RETURNING *",
    )
    .bind(&new_api_key)
    .bind(key_id)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    // Invalidate old key's cache
    crate::cache::invalidate_group_config(&state.redis, &old_key.api_key).await;
    Ok(Json(key))
}
