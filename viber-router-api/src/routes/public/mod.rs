pub mod ttft;
pub mod uptime;
pub mod usage;
pub mod user_endpoints;

use axum::Router;

use crate::routes::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/usage", axum::routing::get(usage::public_usage))
        .route("/ttft", axum::routing::get(ttft::public_ttft))
        .route("/uptime", axum::routing::get(uptime::public_uptime))
        .route(
            "/user-endpoints",
            axum::routing::get(user_endpoints::list_user_endpoints)
                .post(user_endpoints::create_user_endpoint),
        )
        .route(
            "/user-endpoints/{id}",
            axum::routing::patch(user_endpoints::patch_user_endpoint)
                .delete(user_endpoints::delete_user_endpoint),
        )
}
