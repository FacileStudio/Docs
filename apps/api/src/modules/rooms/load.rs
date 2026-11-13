use sqlx::PgPool;
use sqlx::types::Json;
use uuid::Uuid;
use yrs::updates::decoder::Decode;
use yrs::{Doc, ReadTxn, StateVector, Transact, Update};

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
    Migrated(content::Prepared, Vec<u8>, Loaded),
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
    if !ydoc.is_empty() && schema_version == encre::SCHEMA_VERSION as i32 {
        return Ok(Stored::Ready(Loaded { doc: from_state(&ydoc)?, version, trashed }));
    }
    if !ydoc.is_empty() {
        return migrate(&ydoc, schema_version, version, trashed);
    }
    let state = encre::yjs::to_update(&content::load(&stored.0, schema_version)?)?;
    let doc = from_state(&state)?;
    Ok(Stored::Built(state, Loaded { doc, version, trashed }))
}

fn migrate(ydoc: &[u8], schema_version: i32, version: i64, trashed: bool) -> Result<Stored, LoadError> {
    let doc = from_state(ydoc)?;
    let fragment = doc.get_or_insert_xml_fragment(encre::yjs::FRAGMENT);
    let held = encre::yjs::read(&doc.transact(), &fragment)?;
    let node = encre::migrate(held.clone(), u32::try_from(schema_version).unwrap_or(0))?;
    if node != held {
        encre::yjs::write(&mut doc.transact_mut(), &fragment, &node)?;
    }
    let prepared = content::measure(node)?;
    let state = doc.transact().encode_state_as_update_v1(&StateVector::default());
    Ok(Stored::Migrated(prepared, state, Loaded { doc, version, trashed }))
}

async fn stamp(
    pool: &PgPool,
    id: Uuid,
    prepared: &content::Prepared,
    state: &[u8],
    version: i64,
) -> Result<bool, LoadError> {
    let done = sqlx::query(
        "UPDATE documents SET ydoc = $2, content = $3, text = $4, schema_version = $5 WHERE id = $1 AND version = $6",
    )
    .bind(id)
    .bind(state)
    .bind(Json(&prepared.json))
    .bind(&prepared.text)
    .bind(encre::SCHEMA_VERSION as i32)
    .bind(version)
    .execute(pool)
    .await?;
    Ok(done.rows_affected() == 1)
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
/// A stored `ydoc` from an older schema is migrated inside that same Yjs document and stored
/// with the current `schema_version`.
pub async fn load(pool: &PgPool, id: Uuid) -> Result<Option<Loaded>, LoadError> {
    for _ in 0..3 {
        match read(pool, id).await? {
            Stored::Missing => return Ok(None),
            Stored::Ready(loaded) => return Ok(Some(loaded)),
            Stored::Built(state, loaded) if seed(pool, id, &state).await? => return Ok(Some(loaded)),
            Stored::Migrated(prepared, state, loaded) if stamp(pool, id, &prepared, &state, loaded.version).await? => {
                return Ok(Some(loaded));
            }
            Stored::Built(..) | Stored::Migrated(..) => continue,
        }
    }
    Err("the document's Yjs state kept changing while it loaded".into())
}
