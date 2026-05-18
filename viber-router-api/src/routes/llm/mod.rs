pub mod setup;

use axum::Router;

use crate::routes::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/setup-claudecode",
            axum::routing::get(setup::setup_claudecode),
        )
        .route("/setup-codex", axum::routing::get(setup::setup_codex))
}
