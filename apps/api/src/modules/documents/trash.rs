use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access};
use super::service::{answer, fetch};
use super::types::Document;
use crate::extract::db;
use crate::state::AppState;

macro_rules! subtree {
    () => {
        "WITH RECURSIVE sub(id) AS (SELECT id FROM documents WHERE id = $1 \
         UNION SELECT d.id FROM documents d JOIN sub ON d.parent_id = sub.id) "
    };
}

/// `DELETE /documents/{id}`: moves the page and its subtree to the trash.
pub async fn trash(state: &AppState, user_id: i64, id: Uuid) -> Result<()> {
    access::check(state, user_id, id, Access::Manager).await?;
    sqlx::query(concat!(
        subtree!(),
        "UPDATE documents SET trashed_at = now() WHERE id IN (SELECT id FROM sub) AND trashed_at IS NULL"
    ))
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(db)?;
    Ok(())
}

/// `POST /documents/{id}/restore`: brings back the page and what was trashed with it. A page
/// whose parent stays in the trash moves to the root.
pub async fn restore(state: &AppState, user_id: i64, id: Uuid) -> Result<Document> {
    let have = access::check(state, user_id, id, Access::Manager).await?;
    let row = fetch(state, id).await?;
    let Some(trashed_at) = row.trashed_at else { return Err(Error::failed("the document is not in the trash")) };
    let mut tx = state.pool.begin().await.map_err(db)?;
    sqlx::query(concat!(
        subtree!(),
        "UPDATE documents SET trashed_at = NULL WHERE id IN (SELECT id FROM sub) AND trashed_at = $2"
    ))
    .bind(id)
    .bind(trashed_at)
    .execute(&mut *tx)
    .await
    .map_err(db)?;
    sqlx::query(
        "UPDATE documents d SET parent_id = NULL FROM documents p \
         WHERE d.id = $1 AND p.id = d.parent_id AND p.trashed_at IS NOT NULL",
    )
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(db)?;
    tx.commit().await.map_err(db)?;
    answer(fetch(state, id).await?, have, false)
}

/// `DELETE /documents/{id}/permanent`: deletes a trashed page and its subtree for good.
pub async fn permanent(state: &AppState, user_id: i64, id: Uuid) -> Result<()> {
    access::check(state, user_id, id, Access::Manager).await?;
    let deleted = sqlx::query("DELETE FROM documents WHERE id = $1 AND trashed_at IS NOT NULL")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db)?;
    match deleted.rows_affected() {
        0 => Err(Error::failed("only a document in the trash can be deleted")),
        _ => Ok(()),
    }
}
