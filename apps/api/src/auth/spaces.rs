use porte::directory::{Apply, DirectoryError, Snapshot, Space};
use sqlx::{PgConnection, PgPool};

use super::retire::{refuse_empty, retire};
use crate::modules::rooms::{Command, Hub};

/// Applies Registre's directory snapshot to `spaces` and `space_members`, then closes the room
/// connections of people who lost access.
#[derive(Clone)]
pub struct SpacesApply {
    pool: PgPool,
    hub: Hub,
}

impl SpacesApply {
    /// An applier over `pool` that evicts from `hub`.
    pub fn new(pool: PgPool, hub: Hub) -> Self {
        SpacesApply { pool, hub }
    }
}

async fn dropped(tx: &mut PgConnection, snapshot: &Snapshot) -> Result<Vec<i64>, sqlx::Error> {
    let (spaces, subjects): (Vec<String>, Vec<String>) = snapshot
        .spaces
        .iter()
        .flat_map(|space| space.members.iter().map(|m| (space.id.clone(), m.subject.clone())))
        .unzip();
    sqlx::query_scalar(
        "SELECT DISTINCT i.user_id FROM space_members m JOIN porte_identities i ON i.subject = m.subject \
         WHERE (m.space_id, m.subject) NOT IN (SELECT * FROM unnest($1::text[], $2::text[]))",
    )
    .bind(&spaces)
    .bind(&subjects)
    .fetch_all(&mut *tx)
    .await
}

fn failed(error: sqlx::Error) -> DirectoryError {
    DirectoryError::Failed(error.to_string())
}

async fn replace_space(tx: &mut PgConnection, space: &Space) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO spaces (id, name, updated_at) VALUES ($1, $2, now())
         ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, updated_at = now(), deleted_at = NULL",
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

/// What one applied snapshot changed for the live side: who lost access, which pages went to
/// the trash.
#[derive(Debug, Default)]
struct Applied {
    lost: Vec<i64>,
    trashed: Vec<uuid::Uuid>,
}

async fn apply_in(tx: &mut PgConnection, snapshot: &Snapshot) -> Result<Option<Applied>, sqlx::Error> {
    let stored: Option<i64> = sqlx::query_scalar("SELECT version FROM directory_state WHERE id = 1 FOR UPDATE")
        .fetch_optional(&mut *tx)
        .await?;
    if stored.is_some_and(|version| snapshot.version <= version) {
        return Ok(None);
    }
    let lost = dropped(tx, snapshot).await?;
    let ids: Vec<String> = snapshot.spaces.iter().map(|space| space.id.clone()).collect();
    let trashed = retire(tx, &ids).await?;
    for space in &snapshot.spaces {
        replace_space(tx, space).await?;
    }
    sqlx::query(
        "INSERT INTO directory_state (id, version) VALUES (1, $1) ON CONFLICT (id) DO UPDATE SET version = EXCLUDED.version",
    )
    .bind(snapshot.version)
    .execute(&mut *tx)
    .await?;
    Ok(Some(Applied { lost, trashed }))
}

impl SpacesApply {
    /// Applies `snapshot` in one transaction; an older or equal version changes nothing. A
    /// space missing from it is marked deleted and its pages go to the trash, where the 30-day
    /// purge alone deletes them; a space that comes back is undeleted, its pages stay in the
    /// trash. An empty snapshot while spaces exist is refused. Answers whether it applied.
    pub async fn apply_snapshot(&self, snapshot: &Snapshot) -> Result<bool, DirectoryError> {
        refuse_empty(&self.pool, snapshot).await?;
        let mut tx = self.pool.begin().await.map_err(failed)?;
        let applied = apply_in(&mut tx, snapshot).await.map_err(failed)?;
        tx.commit().await.map_err(failed)?;
        let Some(applied) = applied else { return Ok(false) };
        self.hub.close(&applied.trashed).await;
        if !applied.lost.is_empty() {
            self.hub.tell_all(|| Command::Evict(applied.lost.clone())).await;
        }
        Ok(true)
    }
}

#[async_trait::async_trait]
impl Apply for SpacesApply {
    async fn apply(&self, snapshot: &Snapshot) -> Result<(), DirectoryError> {
        self.apply_snapshot(snapshot).await.map(|_| ())
    }
}
