use serde::Serialize;
use time::OffsetDateTime;
use tronc::{Error, Result};
use uuid::Uuid;

use crate::extract::db;
use crate::modules::documents::access::{self, Access};
use crate::state::AppState;

/// The most hits one search answers.
pub const MAX_HITS: i64 = 50;

/// One search hit.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Hit {
    pub id: Uuid,
    pub title: String,
    pub snippet: String,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// Searches the scope `space_id` (personal when `None`). The scope filter is in the same
/// statement as the match, so a page outside it can never be ranked, let alone returned.
pub async fn search(state: &AppState, user_id: i64, q: &str, space_id: Option<&str>) -> Result<Vec<Hit>> {
    access::require(access::scope(state, user_id, space_id).await?, Access::Viewer)?;
    if q.trim().is_empty() {
        return Err(Error::invalid("q is required"));
    }
    sqlx::query_as::<_, Hit>(
        "SELECT d.id, d.title, ts_headline('simple', d.title || ' ' || d.text, query, \
         'MaxFragments=1, MaxWords=24, MinWords=8') AS snippet, d.updated_at \
         FROM documents d, websearch_to_tsquery('simple', $1) query \
         WHERE d.search @@ query AND d.trashed_at IS NULL \
         AND d.space_id IS NOT DISTINCT FROM $2 AND (d.space_id IS NOT NULL OR d.owner_id = $3) \
         ORDER BY ts_rank(d.search, query) DESC, d.updated_at DESC LIMIT $4",
    )
    .bind(q)
    .bind(space_id)
    .bind(user_id)
    .bind(MAX_HITS)
    .fetch_all(&state.pool)
    .await
    .map_err(db)
}
