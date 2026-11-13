use axum::Router;
use axum::extract::State;
use axum::routing::get;
use porte::Identity;
use tronc::json::Json;

use super::{Assistant, assistant};
use crate::state::AppState;

async fn show(State(state): State<AppState>, who: Identity) -> Json<Assistant> {
    Json(assistant(&state, who.user_id).await)
}

/// `GET /assistant`, behind `require_auth`. It needs no plan: it never answers `402`.
pub fn routes() -> Router<AppState> {
    Router::new().route("/assistant", get(show))
}
