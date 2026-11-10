use serde::Serialize;
use serde_json::Value;
use sqlx::types::Json;
use time::OffsetDateTime;
use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access};
use super::content;
use super::service::{answer, fetch};
use super::types::Document;
use crate::extract::db;
use crate::modules::rooms::Command;
use crate::state::AppState;

/// The most revisions one list answers.
pub const MAX_REVISIONS: i64 = 50;

/// One revision, newest first in a list.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Revision {
    pub id: Uuid,
    pub title: String,
    pub schema_version: i32,
    pub content: Json<Value>,
    pub created_by: Option<i64>,
    pub author: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// `GET /documents/{id}/revisions`: the newest [`MAX_REVISIONS`], with their content.
pub async fn list(state: &AppState, user_id: i64, id: Uuid) -> Result<Vec<Revision>> {
    access::check(state, user_id, id, Access::Viewer).await?;
    sqlx::query_as::<_, Revision>(
        "SELECT r.id, r.title, r.schema_version, r.content, r.created_by, u.name AS author, r.created_at \
         FROM document_revisions r LEFT JOIN porte_users u ON u.id = r.created_by \
         WHERE r.document_id = $1 ORDER BY r.created_at DESC, r.id LIMIT $2",
    )
    .bind(id)
    .bind(MAX_REVISIONS)
    .fetch_all(&state.pool)
    .await
    .map_err(db)
}

/// The content of revision `rev` of document `id`, validated as a write.
pub async fn load(state: &AppState, id: Uuid, rev: Uuid) -> Result<content::Prepared> {
    let row: Option<(Json<Value>, i32)> =
        sqlx::query_as("SELECT content, schema_version FROM document_revisions WHERE id = $1 AND document_id = $2")
            .bind(rev)
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(db)?;
    let (stored, schema_version) = row.ok_or_else(|| Error::not_found("revision not found"))?;
    content::prepare(content::load(&stored.0, schema_version)?)
}

/// `POST /documents/{id}/revisions/{rev}/restore`: the document takes the revision's content.
pub async fn restore(state: &AppState, user_id: i64, id: Uuid, rev: Uuid) -> Result<Document> {
    let have = access::check(state, user_id, id, Access::Editor).await?;
    if fetch(state, id).await?.trashed_at.is_some() {
        return Err(Error::failed("the document is in the trash"));
    }
    let prepared = Box::new(load(state, id, rev).await?);
    state.hub.ask(state, id, |reply| Command::Replace(prepared.clone(), user_id, reply)).await?;
    answer(fetch(state, id).await?, have, false)
}
