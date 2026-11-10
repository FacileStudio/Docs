use sqlx::PgPool;
use sqlx::types::Json;
use uuid::Uuid;
use yrs::updates::decoder::Decode;
use yrs::{Doc, Transact, Update};

use crate::modules::documents::content;

/// The state a room starts from.
pub struct Loaded {
    /// The Yjs document, `default` fragment filled.
    pub doc: Doc,
    /// The stored version.
    pub version: i64,
    /// Whether the document is in the trash: everyone is a viewer.
    pub trashed: bool,
}

type LoadError = Box<dyn std::error::Error + Send + Sync>;
type Row = (Vec<u8>, Json<serde_json::Value>, i32, i64, bool);

enum Stored {
    Missing,
    Ready(Loaded),
    Built(Vec<u8>, Loaded),
}

/// A fresh document holding `state`, a full v1 update.
pub fn from_state(state: &[u8]) -> Result<Doc, LoadError> {
    let doc = encre::yjs::new_doc();
    doc.get_or_insert_xml_fragment(encre::yjs::FRAGMENT);
    doc.transact_mut().apply_update(Update::decode_v1(state)?)?;
    Ok(doc)
}

async fn read(pool: &PgPool, id: Uuid) -> Result<Stored, LoadError> {
    let row: Option<Row> = sqlx::query_as(
        "SELECT ydoc, content, schema_version, version, trashed_at IS NOT NULL FROM documents WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let Some((ydoc, stored, schema_version, version, trashed)) = row else { return Ok(Stored::Missing) };
    if !ydoc.is_empty() {
        return Ok(Stored::Ready(Loaded { doc: from_state(&ydoc)?, version, trashed }));
    }
    let state = encre::yjs::to_update(&content::load(&stored.0, schema_version)?)?;
    let doc = from_state(&state)?;
    Ok(Stored::Built(state, Loaded { doc, version, trashed }))
}

async fn seed(pool: &PgPool, id: Uuid, state: &[u8]) -> Result<bool, LoadError> {
    let done = sqlx::query("UPDATE documents SET ydoc = $2 WHERE id = $1 AND ydoc = ''::bytea")
        .bind(id)
        .bind(state)
        .execute(pool)
        .await?;
    Ok(done.rows_affected() == 1)
}

/// Reads `documents.ydoc`, or builds it from `content` when it is empty (a page written by
/// the API or an import) and stores it at once: every later load, after a crash too, must
/// start from the Yjs items the clients already hold, or their state would duplicate blocks.
pub async fn load(pool: &PgPool, id: Uuid) -> Result<Option<Loaded>, LoadError> {
    for _ in 0..3 {
        match read(pool, id).await? {
            Stored::Missing => return Ok(None),
            Stored::Ready(loaded) => return Ok(Some(loaded)),
            Stored::Built(state, loaded) if seed(pool, id, &state).await? => return Ok(Some(loaded)),
            Stored::Built(..) => continue,
        }
    }
    Err("the document's Yjs state kept changing while it loaded".into())
}
