use serde::Serialize;
use tronc::{Error, Result};
use uuid::Uuid;

use super::{files, sniff};
use crate::extract::db;
use crate::modules::documents::access::{self, Access};
use crate::modules::documents::service::fetch;
use crate::modules::share::service as share;
use crate::state::AppState;

/// What an upload answers: `src` (and its older name `url`) is what an `image` or `file`
/// block's `src` holds.
#[derive(Debug, Serialize)]
pub struct Uploaded {
    pub id: Uuid,
    pub url: String,
    pub src: String,
    pub name: String,
    pub size: i64,
    pub mime: String,
}

/// One file sent to `POST /documents/{id}/attachments`.
pub struct Upload<'a> {
    /// The file name the browser sent.
    pub name: &'a str,
    /// The `Content-Type` of the multipart part, if any.
    pub declared: Option<&'a str>,
    /// The bytes.
    pub bytes: &'a [u8],
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

/// The stored type of `file`: the sniffed one for an image or a PDF of at most 20 MiB, else
/// the sanitized declared one for any file within `max` bytes.
pub fn typed(file: &Upload<'_>, max: usize) -> Result<String> {
    let mib = |bytes: usize| bytes >> 20;
    match sniff::sniff(file.bytes) {
        Some(_) if file.bytes.len() > sniff::MAX_BYTES => {
            Err(Error::too_large(format!("an image or a PDF is at most {} MiB", mib(sniff::MAX_BYTES))))
        }
        Some(mime) => Ok(mime.to_owned()),
        None if file.bytes.len() > max => Err(Error::too_large(format!("a file is at most {} MiB", mib(max)))),
        None => Ok(sniff::declared(file.declared)),
    }
}

/// `POST /documents/{id}/attachments`: stores the bytes once. Images and PDFs are typed by
/// their magic number; any other file keeps its declared type, sanitized, and is served as a
/// download.
pub async fn upload(state: &AppState, user_id: i64, document: Uuid, file: Upload<'_>) -> Result<Uploaded> {
    access::check(state, user_id, document, Access::Editor).await?;
    if fetch(state, document).await?.trashed_at.is_some() {
        return Err(Error::failed("the document is in the trash"));
    }
    let mime = typed(&file, state.attachment_max)?;
    let (name, bytes) = (clean_name(file.name), file.bytes);
    let sha256 =
        files::store(&state.data_dir, bytes).await.map_err(|e| Error::internal("failed to store a file", e))?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO attachments (document_id, name, mime, size, sha256, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(document)
    .bind(&name)
    .bind(&mime)
    .bind(bytes.len() as i64)
    .bind(&sha256)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    Ok(Uploaded { id, url: url(id), src: url(id), name, size: bytes.len() as i64, mime })
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
