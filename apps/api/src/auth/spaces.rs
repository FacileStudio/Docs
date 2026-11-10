use porte::directory::{Apply, DirectoryError, Snapshot, Space};
use sqlx::{PgConnection, PgPool};

/// Applies Registre's directory snapshot to `spaces` and `space_members`.
#[derive(Clone)]
pub struct SpacesApply {
    pool: PgPool,
}

impl SpacesApply {
    /// An applier over `pool`.
    pub fn new(pool: PgPool) -> Self {
        SpacesApply { pool }
    }
}

fn failed(error: sqlx::Error) -> DirectoryError {
    DirectoryError::Failed(error.to_string())
}

async fn replace_space(tx: &mut PgConnection, space: &Space) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO spaces (id, name, updated_at) VALUES ($1, $2, now())
         ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, updated_at = now()",
    )
    .bind(&space.id)
    .bind(&space.name)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM space_members WHERE space_id = $1").bind(&space.id).execute(&mut *tx).await?;
    for member in &space.members {
        sqlx::query(
            "INSERT INTO space_members (space_id, subject, email, name, picture, role)
             VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (space_id, subject) DO NOTHING",
        )
        .bind(&space.id)
        .bind(&member.subject)
        .bind(&member.email)
        .bind(&member.name)
        .bind(&member.picture)
        .bind(&member.role)
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}

async fn apply_in(tx: &mut PgConnection, snapshot: &Snapshot) -> Result<bool, sqlx::Error> {
    let stored: Option<i64> = sqlx::query_scalar("SELECT version FROM directory_state WHERE id = 1 FOR UPDATE")
        .fetch_optional(&mut *tx)
        .await?;
    if stored.is_some_and(|version| snapshot.version <= version) {
        return Ok(false);
    }
    let ids: Vec<String> = snapshot.spaces.iter().map(|space| space.id.clone()).collect();
    sqlx::query("DELETE FROM spaces WHERE id <> ALL($1)").bind(&ids).execute(&mut *tx).await?;
    for space in &snapshot.spaces {
        replace_space(tx, space).await?;
    }
    sqlx::query(
        "INSERT INTO directory_state (id, version) VALUES (1, $1) ON CONFLICT (id) DO UPDATE SET version = EXCLUDED.version",
    )
    .bind(snapshot.version)
    .execute(&mut *tx)
    .await?;
    Ok(true)
}

impl SpacesApply {
    /// Applies `snapshot` in one transaction; an older or equal version changes nothing.
    pub async fn apply_snapshot(&self, snapshot: &Snapshot) -> Result<bool, DirectoryError> {
        let mut tx = self.pool.begin().await.map_err(failed)?;
        let changed = apply_in(&mut tx, snapshot).await.map_err(failed)?;
        tx.commit().await.map_err(failed)?;
        Ok(changed)
    }
}

#[async_trait::async_trait]
impl Apply for SpacesApply {
    async fn apply(&self, snapshot: &Snapshot) -> Result<(), DirectoryError> {
        self.apply_snapshot(snapshot).await.map(|_| ())
    }
}
