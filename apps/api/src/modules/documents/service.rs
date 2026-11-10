use encre::Node;
use sqlx::types::Json;
use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access};
use super::content::{self, Prepared};
use super::types::{Body, CreateRequest, Document, DocumentRow, PatchRequest, Stamps, columns};
use crate::extract::db;
use crate::state::AppState;

/// The longest title, in characters.
pub const MAX_TITLE: usize = 200;

/// Reads one row, whoever asks: callers check access first.
pub async fn fetch(state: &AppState, id: Uuid) -> Result<DocumentRow> {
    sqlx::query_as::<_, DocumentRow>(concat!("SELECT ", columns!(), " FROM documents WHERE id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(db)?
        .ok_or_else(|| Error::not_found("document not found"))
}

/// The row as the API answers it for a caller with `access`, with the body in `content` or in
/// `markdown`.
pub fn answer(row: DocumentRow, access: Access, markdown: bool) -> Result<Document> {
    let body = match markdown {
        true => Body::Markdown(encre::markdown::serialize(&content::load(&row.content.0, row.schema_version)?)),
        false => Body::Content(row.content.0),
    };
    Ok(Document {
        id: row.id,
        space_id: row.space_id,
        parent_id: row.parent_id,
        position: row.position,
        title: row.title,
        icon: row.icon,
        schema_version: row.schema_version,
        version: row.version,
        stamps: Stamps { created_at: row.created_at, updated_at: row.updated_at, trashed_at: row.trashed_at },
        access: access.name(),
        body,
    })
}

/// A trimmed title of at most [`MAX_TITLE`] characters.
pub fn title(raw: &str) -> Result<String> {
    let title = raw.trim();
    match title.chars().count() > MAX_TITLE {
        true => Err(Error::invalid(format!("title is longer than {MAX_TITLE} characters"))),
        false => Ok(title.to_owned()),
    }
}

async fn placement(state: &AppState, user_id: i64, request: &CreateRequest) -> Result<(Option<String>, Access)> {
    let Some(parent) = request.parent_id else {
        let have = access::scope(state, user_id, request.space_id.as_deref()).await?;
        access::require(have, Access::Editor)?;
        return Ok((request.space_id.clone(), have));
    };
    let have = access::check(state, user_id, parent, Access::Editor).await?;
    let parent = fetch(state, parent).await?;
    if parent.trashed_at.is_some() {
        return Err(Error::failed("the parent document is in the trash"));
    }
    if request.space_id.is_some() && request.space_id != parent.space_id {
        return Err(Error::invalid("space_id differs from the parent's space"));
    }
    Ok((parent.space_id, have))
}

/// `POST /documents`: a new page at the end of its siblings.
pub async fn create(state: &AppState, user_id: i64, request: CreateRequest) -> Result<Document> {
    let (space_id, have) = placement(state, user_id, &request).await?;
    let title = title(&request.title)?;
    let node: Node = content::from_input(request.markdown, request.content, state.markdown_max).await?;
    let Prepared { json, text, .. } = content::prepare(node)?;
    let row = sqlx::query_as::<_, DocumentRow>(concat!(
        "INSERT INTO documents (space_id, owner_id, parent_id, position, title, schema_version, content, text, \
         created_by, updated_by) SELECT $1, $2, $3, coalesce(max(position), 0) + 1, $4, $5, $6, $7, $2, $2 \
         FROM documents WHERE space_id IS NOT DISTINCT FROM $1 AND parent_id IS NOT DISTINCT FROM $3 \
         AND (space_id IS NOT NULL OR owner_id = $2) AND trashed_at IS NULL RETURNING ",
        columns!()
    ))
    .bind(&space_id)
    .bind(user_id)
    .bind(request.parent_id)
    .bind(title)
    .bind(encre::SCHEMA_VERSION as i32)
    .bind(Json(json))
    .bind(text)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    answer(row, have, false)
}

/// `PATCH /documents/{id}`: title and icon.
pub async fn patch(state: &AppState, user_id: i64, id: Uuid, request: PatchRequest) -> Result<Document> {
    let have = access::check(state, user_id, id, Access::Editor).await?;
    let title = request.title.as_deref().map(title).transpose()?;
    if request.icon.as_ref().is_some_and(|icon| icon.chars().count() > 32) {
        return Err(Error::invalid("icon is longer than 32 characters"));
    }
    let row = sqlx::query_as::<_, DocumentRow>(concat!(
        "UPDATE documents SET title = coalesce($2, title), icon = coalesce($3, icon), updated_at = now(), \
         updated_by = $4 WHERE id = $1 RETURNING ",
        columns!()
    ))
    .bind(id)
    .bind(title)
    .bind(request.icon)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    answer(row, have, false)
}

#[cfg(test)]
mod tests {
    #[test]
    fn titles_are_trimmed_and_bounded() {
        assert_eq!(super::title("  Plan  ").unwrap(), "Plan");
        assert!(super::title(&"x".repeat(201)).is_err());
    }
}
