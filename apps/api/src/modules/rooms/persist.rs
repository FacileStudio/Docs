use sqlx::PgPool;
use uuid::Uuid;
use yrs::{Doc, ReadTxn, StateVector, Transact};

use crate::extract::db;
use crate::modules::documents::content::{self, Prepared};
use crate::modules::documents::write;

/// The document as it stands in the room, validated as a stored body.
pub fn snapshot(doc: &Doc) -> tronc::Result<(Prepared, Vec<u8>)> {
    let fragment = doc.get_or_insert_xml_fragment(encre::yjs::FRAGMENT);
    let txn = doc.transact();
    let node = encre::yjs::read(&txn, &fragment).map_err(|e| tronc::Error::internal("unreadable room", e))?;
    Ok((content::measure(node)?, txn.encode_state_as_update_v1(&StateVector::default())))
}

/// Writes `ydoc`, `content`, `text` and a new version in one statement, plus a revision when
/// none was written in ten minutes or `force` is set. Answers the new version, or `NotFound`
/// when the row is gone.
pub async fn persist(pool: &PgPool, id: Uuid, doc: &Doc, author: Option<i64>, force: bool) -> tronc::Result<i64> {
    let (prepared, ydoc) = snapshot(doc)?;
    let mut tx = pool.begin().await.map_err(db)?;
    let version = write::save(&mut tx, id, &prepared, &ydoc, author).await.map_err(|error| match error {
        sqlx::Error::RowNotFound => tronc::Error::not_found("the document was deleted"),
        other => db(other),
    })?;
    write::revision(&mut tx, id, author, force).await.map_err(db)?;
    tx.commit().await.map_err(db)?;
    Ok(version)
}
