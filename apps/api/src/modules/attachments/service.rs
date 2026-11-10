use serde::Serialize;
use tronc::{Error, Result};
use uuid::Uuid;

use super::{files, sniff};
use crate::extract::db;
use crate::modules::documents::access::{self, Access};
use crate::modules::documents::service::fetch;
use crate::modules::share::service as share;
use crate::state::AppState;

/// What an upload answers: `url` is what an `image` block's `src` holds.
#[derive(Debug, Serialize)]
pub struct Uploaded {
    pub id: Uuid,
    pub url: String,
}

/// One stored attachment.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Attachment {
    pub id: Uuid,
    pub document_id: Uuid,
    pub name: String,
    pub mime: String,
    pub sha256: String,
}

/// The attachment URL for `id`.
pub fn url(id: Uuid) -> String {
    format!("/api/attachments/{id}")
}

/// A file name safe for a header: no path, no control characters, at most 200 characters.
pub fn clean_name(raw: &str) -> String {
    let base = raw.rsplit(['/', '\\']).next().unwrap_or_default();
    let name: String = base.chars().filter(|c| !c.is_control() && *c != '"').take(200).collect();
    if name.trim().is_empty() { "file".to_owned() } else { name }
}

/// `POST /documents/{id}/attachments`: stores the bytes once, typed by their magic number.
pub async fn upload(state: &AppState, user_id: i64, document: Uuid, name: &str, bytes: &[u8]) -> Result<Uploaded> {
    access::check(state, user_id, document, Access::Editor).await?;
    if fetch(state, document).await?.trashed_at.is_some() {
        return Err(Error::failed("the document is in the trash"));
    }
    if bytes.len() > sniff::MAX_BYTES {
        return Err(Error::too_large("an attachment is at most 20 MiB"));
    }
    let mime =
        sniff::sniff(bytes).ok_or_else(|| Error::invalid("unsupported file type: send PNG, JPEG, GIF, WebP or PDF"))?;
    let sha256 =
        files::store(&state.data_dir, bytes).await.map_err(|e| Error::internal("failed to store a file", e))?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO attachments (document_id, name, mime, size, sha256, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(document)
    .bind(clean_name(name))
    .bind(mime)
    .bind(bytes.len() as i64)
    .bind(&sha256)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    Ok(Uploaded { id, url: url(id) })
}

/// The attachment row, whoever asks.
pub async fn find(state: &AppState, id: Uuid) -> Result<Attachment> {
    sqlx::query_as::<_, Attachment>("SELECT id, document_id, name, mime, sha256 FROM attachments WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(db)?
        .ok_or_else(|| Error::not_found("attachment not found"))
}

/// The attachment and its bytes, for a viewer of its document or a share link that covers it.
pub async fn open(state: &AppState, user: Option<i64>, token: Option<&str>, id: Uuid) -> Result<(Attachment, Vec<u8>)> {
    let attachment = find(state, id).await?;
    let allowed = match (token, user) {
        (Some(token), _) => share::covers(state, &share::resolve(state, token).await?, attachment.document_id).await?,
        (None, Some(user)) => access::access(state, user, attachment.document_id).await? >= Access::Viewer,
        (None, None) => return Err(Error::unauthorized("missing auth")),
    };
    if !allowed {
        return Err(Error::not_found("attachment not found"));
    }
    let bytes = tokio::fs::read(files::path(&state.data_dir, &attachment.sha256))
        .await
        .map_err(|error| Error::internal("an attachment file is missing", error))?;
    Ok((attachment, bytes))
}

#[cfg(test)]
mod tests {
    #[test]
    fn names_lose_paths_quotes_and_controls() {
        assert_eq!(super::clean_name("../../etc/pa\"ss\nwd"), "passwd");
        assert_eq!(super::clean_name("C:\\a\\b.png"), "b.png");
        assert_eq!(super::clean_name(""), "file");
    }
}
