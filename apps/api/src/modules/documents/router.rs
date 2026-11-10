use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use porte::Identity;
use tronc::Result;
use tronc::json::Json;

use super::handlers::{apply, create, list, move_to, patch, read, remove, restore};
use super::revisions::{self, Revision};
use super::trash;
use super::types::Document;
use crate::extract::id;
use crate::state::AppState;

async fn permanent(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<StatusCode> {
    trash::permanent(&state, who.user_id, id(&raw)?).await.map(|()| StatusCode::NO_CONTENT)
}

async fn history(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Vec<Revision>>> {
    revisions::list(&state, who.user_id, id(&raw)?).await.map(Json)
}

async fn rollback(
    State(state): State<AppState>,
    who: Identity,
    Path((raw, rev)): Path<(String, String)>,
) -> Result<Json<Document>> {
    revisions::restore(&state, who.user_id, id(&raw)?, id(&rev)?).await.map(Json)
}

/// The document routes, mounted under `/api` behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/documents", get(list).post(create))
        .route("/documents/{id}", get(read).patch(patch).delete(remove))
        .route("/documents/{id}/ops", post(apply))
        .route("/documents/{id}/move", post(move_to))
        .route("/documents/{id}/restore", post(restore))
        .route("/documents/{id}/permanent", delete(permanent))
        .route("/documents/{id}/revisions", get(history))
        .route("/documents/{id}/revisions/{rev}/restore", post(rollback))
}
