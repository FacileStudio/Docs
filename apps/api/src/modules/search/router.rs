use axum::Router;
use axum::extract::State;
use axum::routing::get;
use porte::Identity;
use serde::Deserialize;
use tronc::Result;
use tronc::json::Json;

use super::service::{Hit, search};
use crate::extract::Params;
use crate::state::AppState;

#[derive(Deserialize)]
struct Query {
    #[serde(default)]
    q: String,
    space_id: Option<String>,
}

async fn find(State(state): State<AppState>, who: Identity, Params(query): Params<Query>) -> Result<Json<Vec<Hit>>> {
    search(&state, who.user_id, &query.q, query.space_id.as_deref()).await.map(Json)
}

/// `GET /search`, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new().route("/search", get(find))
}
