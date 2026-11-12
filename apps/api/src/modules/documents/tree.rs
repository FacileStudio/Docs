use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access};
use super::service::{answer, fetch};
use super::types::{Document, MoveRequest, ScopeQuery, TreeEntry};
use crate::extract::db;
use crate::state::AppState;

/// `GET /documents`: every page of the scope, flat, parents before positions. No content.
pub async fn list(state: &AppState, user_id: i64, query: &ScopeQuery) -> Result<Vec<TreeEntry>> {
    access::require(access::scope(state, user_id, query.space_id.as_deref()).await?, Access::Viewer)?;
    sqlx::query_as::<_, TreeEntry>(
        "SELECT id, parent_id, position, title, icon, updated_at, trashed_at FROM documents \
         WHERE space_id IS NOT DISTINCT FROM $1 AND (space_id IS NOT NULL OR owner_id = $2) \
         AND (trashed_at IS NOT NULL) = $3 ORDER BY parent_id NULLS FIRST, position, id",
    )
    .bind(&query.space_id)
    .bind(user_id)
    .bind(query.trashed)
    .fetch_all(&state.pool)
    .await
    .map_err(db)
}

/// The position between `before`'s previous sibling and `before`, or after the last sibling.
pub fn position(siblings: &[(Uuid, f64)], before: Option<Uuid>) -> Result<f64> {
    let Some(before) = before else {
        return Ok(siblings.last().map_or(1.0, |(_, last)| last + 1.0));
    };
    let index = siblings
        .iter()
        .position(|(id, _)| *id == before)
        .ok_or_else(|| Error::invalid("before is not a page under the new parent"))?;
    let next = siblings[index].1;
    Ok(match index {
        0 => next - 1.0,
        _ => (siblings[index - 1].1 + next) / 2.0,
    })
}

async fn check_parent(state: &AppState, user_id: i64, id: Uuid, parent: Uuid) -> Result<()> {
    access::check(state, user_id, parent, Access::Editor).await?;
    let (moving, target) = (fetch(state, id).await?, fetch(state, parent).await?);
    if moving.space_id != target.space_id {
        return Err(Error::invalid("a page cannot move to another space"));
    }
    if target.trashed_at.is_some() {
        return Err(Error::failed("the new parent is in the trash"));
    }
    let inside: bool = sqlx::query_scalar(
        "WITH RECURSIVE up(id, parent_id) AS (SELECT id, parent_id FROM documents WHERE id = $1 \
         UNION SELECT d.id, d.parent_id FROM documents d JOIN up ON d.id = up.parent_id) \
         SELECT EXISTS (SELECT 1 FROM up WHERE id = $2)",
    )
    .bind(parent)
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    match inside {
        true => Err(Error::invalid("a page cannot move inside itself")),
        false => Ok(()),
    }
}

async fn siblings(state: &AppState, user_id: i64, id: Uuid, parent: Option<Uuid>) -> Result<Vec<(Uuid, f64)>> {
    let moving = fetch(state, id).await?;
    sqlx::query_as(
        "SELECT id, position FROM documents WHERE space_id IS NOT DISTINCT FROM $1 \
         AND parent_id IS NOT DISTINCT FROM $2 AND (space_id IS NOT NULL OR owner_id = $3) \
         AND trashed_at IS NULL AND id <> $4 ORDER BY position, id",
    )
    .bind(&moving.space_id)
    .bind(parent)
    .bind(user_id)
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(db)
}

/// `POST /documents/{id}/move`: under `parent_id` (the root when absent), before `before` (the
/// end when absent). The subtree follows; another space is refused.
pub async fn move_to(state: &AppState, user_id: i64, id: Uuid, request: MoveRequest) -> Result<Document> {
    let have = access::check(state, user_id, id, Access::Manager).await?;
    if let Some(parent) = request.parent_id {
        check_parent(state, user_id, id, parent).await?;
    }
    let position = position(&siblings(state, user_id, id, request.parent_id).await?, request.before)?;
    let moved = sqlx::query(
        "WITH parent AS (SELECT id FROM documents WHERE id = $2 AND trashed_at IS NULL FOR SHARE) \
         UPDATE documents SET parent_id = $2, position = $3, updated_at = now() WHERE id = $1 \
         AND trashed_at IS NULL AND ($2::uuid IS NULL OR EXISTS (SELECT 1 FROM parent))",
    )
    .bind(id)
    .bind(request.parent_id)
    .bind(position)
    .execute(&state.pool)
    .await
    .map_err(db)?;
    if moved.rows_affected() == 0 {
        return Err(Error::failed("the page or its new parent is in the trash"));
    }
    answer(fetch(state, id).await?, have, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_are_midpoints_of_the_new_neighbours() {
        let (a, b, c) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let siblings = [(a, 1.0), (b, 2.0), (c, 4.0)];
        assert_eq!(position(&siblings, None).unwrap(), 5.0);
        assert_eq!(position(&siblings, Some(a)).unwrap(), 0.0);
        assert_eq!(position(&siblings, Some(c)).unwrap(), 3.0);
        assert_eq!(position(&[], None).unwrap(), 1.0);
        assert!(position(&siblings, Some(Uuid::new_v4())).is_err());
    }
}
