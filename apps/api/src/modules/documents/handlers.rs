//! The document handlers: thin wrappers that read the caller and the path.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use porte::Identity;
use tronc::Result;
use tronc::json::Json;

use super::types::{CreateRequest, Document, FormatQuery, MoveRequest, OpsAnswer, OpsRequest, PatchRequest};
use super::types::{ScopeQuery, TreeEntry};
use super::{access, ops, service, trash, tree};
use crate::extract::{Params, id};
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    who: Identity,
    Params(query): Params<ScopeQuery>,
) -> Result<Json<Vec<TreeEntry>>> {
    tree::list(&state, who.user_id, &query).await.map(Json)
}

pub async fn create(
    State(state): State<AppState>,
    who: Identity,
    Json(body): Json<CreateRequest>,
) -> Result<(StatusCode, Json<Document>)> {
    Ok((StatusCode::CREATED, Json(service::create(&state, who.user_id, body).await?)))
}

pub async fn read(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Params(query): Params<FormatQuery>,
) -> Result<Json<Document>> {
    let id = id(&raw)?;
    let markdown = match query.format.as_deref() {
        None | Some("json") => false,
        Some("markdown") => true,
        Some(_) => return Err(tronc::Error::invalid("format is json or markdown")),
    };
    let have = access::check(&state, who.user_id, id, access::Access::Viewer).await?;
    service::answer(service::fetch(&state, id).await?, have, markdown).map(Json)
}

pub async fn patch(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Json(body): Json<PatchRequest>,
) -> Result<Json<Document>> {
    service::patch(&state, who.user_id, id(&raw)?, body).await.map(Json)
}

pub async fn apply(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Json(body): Json<OpsRequest>,
) -> Result<Json<OpsAnswer>> {
    ops::apply_stored(&state, who.user_id, id(&raw)?, &body.ops).await.map(Json)
}

pub async fn move_to(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Json(body): Json<MoveRequest>,
) -> Result<Json<Document>> {
    tree::move_to(&state, who.user_id, id(&raw)?, body).await.map(Json)
}

pub async fn remove(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<StatusCode> {
    trash::trash(&state, who.user_id, id(&raw)?).await.map(|()| StatusCode::NO_CONTENT)
}

pub async fn restore(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Document>> {
    trash::restore(&state, who.user_id, id(&raw)?).await.map(Json)
}
