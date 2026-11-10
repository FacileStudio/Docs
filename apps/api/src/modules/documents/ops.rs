use encre::ops::{self, Op, OpError};
use sqlx::types::Json;
use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access};
use super::content;
use super::types::OpsAnswer;
use super::write;
use crate::extract::db;
use crate::state::AppState;

/// The most operations one request applies.
pub const MAX_OPS: usize = 100;

/// An operation refusal as the suite answers it: a stale `replace_all` is `412`, the rest `400`.
pub fn refusal(error: OpError) -> Error {
    match error {
        OpError::Version { .. } => Error::failed(error.to_string()),
        _ => Error::invalid(error.to_string()),
    }
}

/// Applies `ops` in order to `doc`, all or nothing. Answers whether anything changed.
pub fn apply_all(doc: &mut encre::Node, version: u64, list: &[Op]) -> Result<bool> {
    if list.len() > MAX_OPS {
        return Err(Error::invalid(format!("at most {MAX_OPS} operations per request")));
    }
    let mut next = doc.clone();
    let mut changed = false;
    for op in list {
        if let ops::Outcome::Changed(_) = ops::apply(&mut next, version, op).map_err(refusal)? {
            changed = true;
        }
    }
    *doc = next;
    Ok(changed)
}

/// `POST /documents/{id}/ops` against the stored JSON: the Yjs state is cleared and the next
/// room rebuilds it from the new content.
pub async fn apply_stored(state: &AppState, user_id: i64, id: Uuid, list: &[Op]) -> Result<OpsAnswer> {
    access::check(state, user_id, id, Access::Editor).await?;
    let mut tx = state.pool.begin().await.map_err(db)?;
    let row: (Json<serde_json::Value>, i32, i64, bool) = sqlx::query_as(
        "SELECT content, schema_version, version, trashed_at IS NOT NULL FROM documents WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db)?;
    if row.3 {
        return Err(Error::failed("the document is in the trash"));
    }
    let mut doc = content::load(&row.0.0, row.1)?;
    let mut version = row.2;
    if apply_all(&mut doc, version as u64, list)? {
        let prepared = content::prepare(doc)?;
        version = write::save(&mut tx, id, &prepared, &[], Some(user_id)).await.map_err(db)?;
        write::revision(&mut tx, id, Some(user_id), false).await.map_err(db)?;
        doc = prepared.node;
    }
    tx.commit().await.map_err(db)?;
    Ok(OpsAnswer { version, markdown: encre::markdown::serialize(&doc) })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(json: serde_json::Value) -> Op {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn a_refused_operation_leaves_the_batch_unapplied() {
        let mut doc = content::prepare(content::from_markdown("one").unwrap()).unwrap().node;
        let before = doc.clone();
        let list = [
            op(serde_json::json!({"op": "append", "markdown": "two"})),
            op(serde_json::json!({"op": "delete_block", "block_id": "0000000000"})),
        ];
        assert_eq!(apply_all(&mut doc, 1, &list).unwrap_err().code, tronc::Code::InvalidArgument);
        assert_eq!(doc, before);
    }

    #[test]
    fn a_stale_replace_all_is_a_precondition_failure() {
        let mut doc = content::prepare(content::from_markdown("one").unwrap()).unwrap().node;
        let list = [op(serde_json::json!({"op": "replace_all", "markdown": "x", "expect_version": 1}))];
        assert_eq!(apply_all(&mut doc, 2, &list).unwrap_err().code, tronc::Code::FailedPrecondition);
    }
}
