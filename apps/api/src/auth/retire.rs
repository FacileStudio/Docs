use porte::directory::{DirectoryError, Snapshot};
use sqlx::{PgConnection, PgPool};

fn failed(error: sqlx::Error) -> DirectoryError {
    DirectoryError::Failed(error.to_string())
}

/// Marks every live space not in `keep` deleted, drops its members and trashes its live pages.
/// Answers the trashed page ids.
pub async fn retire(tx: &mut PgConnection, keep: &[String]) -> Result<Vec<uuid::Uuid>, sqlx::Error> {
    let gone: Vec<String> = sqlx::query_scalar(
        "UPDATE spaces SET deleted_at = now() WHERE id <> ALL($1) AND deleted_at IS NULL RETURNING id",
    )
    .bind(keep)
    .fetch_all(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM space_members WHERE space_id = ANY($1)").bind(&gone).execute(&mut *tx).await?;
    sqlx::query_scalar(
        "UPDATE documents SET trashed_at = now() WHERE space_id = ANY($1) AND trashed_at IS NULL RETURNING id",
    )
    .bind(&gone)
    .fetch_all(&mut *tx)
    .await
}

/// Refuses an empty snapshot while live spaces exist: a short answer from Registre must never
/// retire every space.
pub async fn refuse_empty(pool: &PgPool, snapshot: &Snapshot) -> Result<(), DirectoryError> {
    if !snapshot.spaces.is_empty() {
        return Ok(());
    }
    let live: i64 = sqlx::query_scalar("SELECT count(*) FROM spaces WHERE deleted_at IS NULL")
        .fetch_one(pool)
        .await
        .map_err(failed)?;
    if live == 0 {
        return Ok(());
    }
    tracing::error!(live, version = snapshot.version, "directory: refusing an empty snapshot while spaces exist");
    Err(DirectoryError::Failed(format!("refusing an empty directory snapshot while {live} spaces exist")))
}
