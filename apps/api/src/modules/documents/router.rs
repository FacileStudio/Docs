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
use super::types::{CreateRequest, Document, OpenUiRequest};
use super::{markdown, service};
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

async fn import_openui(
    State(state): State<AppState>,
    who: Identity,
    Json(body): Json<OpenUiRequest>,
) -> Result<(StatusCode, Json<Document>)> {
    markdown::within(body.source.len(), state.markdown_max)?;
    let source = body.source;
    let doc = tokio::task::spawn_blocking(move || encre::openui::to_document(&source))
        .await
        .map_err(|e| tronc::Error::internal("the openui parse did not finish", e))?
        .map_err(|e| tronc::Error::invalid(format!("openui: {e}")))?;
    let content = serde_json::to_value(&doc).map_err(|e| tronc::Error::internal("openui document", e))?;
    let request = CreateRequest {
        title: body.title,
        space_id: body.space_id,
        parent_id: body.parent_id,
        markdown: None,
        content: Some(content),
    };
    Ok((StatusCode::CREATED, Json(service::create(&state, who.user_id, request).await?)))
}

/// The document routes, mounted under `/api` behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/documents", get(list).post(create))
        .route("/documents/{id}", get(read).patch(patch).delete(remove))
        .route("/documents/import-openui", post(import_openui))
        .route("/documents/{id}/ops", post(apply))
        .route("/documents/{id}/move", post(move_to))
        .route("/documents/{id}/restore", post(restore))
        .route("/documents/{id}/permanent", delete(permanent))
        .route("/documents/{id}/revisions", get(history))
        .route("/documents/{id}/revisions/{rev}/restore", post(rollback))
}
