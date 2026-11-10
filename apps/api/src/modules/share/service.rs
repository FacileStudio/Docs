use serde::Serialize;
use time::OffsetDateTime;
use tronc::{Error, Result};
use uuid::Uuid;

use crate::extract::db;
use crate::modules::documents::access::{self, Access};
use crate::state::AppState;

/// A link as `POST /documents/{id}/share` answers it: the token is in `url`, shown once.
#[derive(Debug, Serialize)]
pub struct Created {
    pub id: Uuid,
    pub url: String,
}

/// A live link as the manager's list shows it, without its token.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Link {
    pub id: Uuid,
    pub children: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// A resolved token.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Share {
    pub id: Uuid,
    pub document_id: Uuid,
    pub children: bool,
}

/// `POST /documents/{id}/share`: a new link; only its hash is stored.
pub async fn create(state: &AppState, user_id: i64, document: Uuid, children: bool) -> Result<Created> {
    access::check(state, user_id, document, Access::Manager).await?;
    let token = porte::new_token().map_err(|error| Error::internal("no randomness for a share token", error))?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO share_links (document_id, token_hash, children, created_by) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(document)
    .bind(porte::hash_token(&token))
    .bind(children)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    Ok(Created { id, url: format!("/s/{token}") })
}

/// `GET /documents/{id}/share`: the live links of a page.
pub async fn list(state: &AppState, user_id: i64, document: Uuid) -> Result<Vec<Link>> {
    access::check(state, user_id, document, Access::Manager).await?;
    sqlx::query_as::<_, Link>(
        "SELECT id, children, created_at FROM share_links WHERE document_id = $1 AND revoked_at IS NULL \
         ORDER BY created_at DESC",
    )
    .bind(document)
    .fetch_all(&state.pool)
    .await
    .map_err(db)
}

/// `DELETE /share/{id}`: revokes a link. A link the caller cannot manage is `404`.
pub async fn revoke(state: &AppState, user_id: i64, id: Uuid) -> Result<()> {
    let document: Option<Uuid> =
        sqlx::query_scalar("SELECT document_id FROM share_links WHERE id = $1 AND revoked_at IS NULL")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(db)?;
    let document = document.ok_or_else(|| Error::not_found("share link not found"))?;
    access::check(state, user_id, document, Access::Manager)
        .await
        .map_err(|_| Error::not_found("share link not found"))?;
    sqlx::query("UPDATE share_links SET revoked_at = now() WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db)?;
    Ok(())
}

/// The live link `token` names, on a page that is not in the trash.
pub async fn resolve(state: &AppState, token: &str) -> Result<Share> {
    sqlx::query_as::<_, Share>(
        "SELECT s.id, s.document_id, s.children FROM share_links s JOIN documents d ON d.id = s.document_id \
         WHERE s.token_hash = $1 AND s.revoked_at IS NULL AND d.trashed_at IS NULL",
    )
    .bind(porte::hash_token(token))
    .fetch_optional(&state.pool)
    .await
    .map_err(db)?
    .ok_or_else(|| Error::not_found("share link not found"))
}

/// Whether `share` exposes `document`: its own page, or a live page of its subtree when
/// `children` is on.
pub async fn covers(state: &AppState, share: &Share, document: Uuid) -> Result<bool> {
    if document == share.document_id {
        return Ok(true);
    }
    if !share.children {
        return Ok(false);
    }
    sqlx::query_scalar(
        "WITH RECURSIVE sub(id) AS (SELECT id FROM documents WHERE parent_id = $1 AND trashed_at IS NULL \
         UNION SELECT d.id FROM documents d JOIN sub ON d.parent_id = sub.id WHERE d.trashed_at IS NULL) \
         SELECT EXISTS (SELECT 1 FROM sub WHERE id = $2)",
    )
    .bind(share.document_id)
    .bind(document)
    .fetch_one(&state.pool)
    .await
    .map_err(db)
}
