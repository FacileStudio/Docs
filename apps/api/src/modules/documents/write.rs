use sqlx::PgConnection;
use sqlx::types::Json;
use uuid::Uuid;

use super::content::Prepared;

/// Stores a new body, bumps the version and answers it. `ydoc` is the Yjs state that matches,
/// or empty when the next room must rebuild it from `content`.
pub async fn save(
    conn: &mut PgConnection,
    id: Uuid,
    prepared: &Prepared,
    ydoc: &[u8],
    user_id: Option<i64>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "UPDATE documents SET content = $2, text = $3, ydoc = $4, schema_version = $5, version = version + 1, \
         updated_at = now(), updated_by = coalesce((SELECT u.id FROM porte_users u WHERE u.id = $6), updated_by) \
         WHERE id = $1 RETURNING version",
    )
    .bind(id)
    .bind(Json(&prepared.json))
    .bind(&prepared.text)
    .bind(ydoc)
    .bind(encre::SCHEMA_VERSION as i32)
    .bind(user_id)
    .fetch_one(conn)
    .await
}

/// Writes a revision from the stored body, unless one was written in the last ten minutes and
/// `force` is false.
pub async fn revision(conn: &mut PgConnection, id: Uuid, user_id: Option<i64>, force: bool) -> Result<(), sqlx::Error> {
    sqlx::query(concat!(
        "INSERT INTO document_revisions (document_id, title, schema_version, content, created_by) \
         SELECT id, title, schema_version, content, (SELECT u.id FROM porte_users u WHERE u.id = $2) FROM documents WHERE id = $1 AND ($3 OR NOT EXISTS (\
         SELECT 1 FROM document_revisions r WHERE r.document_id = $1 AND r.created_at > now() - interval '",
        "10 minutes",
        "'))"
    ))
    .bind(id)
    .bind(user_id)
    .bind(force)
    .execute(conn)
    .await
    .map(|_| ())
}
