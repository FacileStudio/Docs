use std::time::Duration;

use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::state::AppState;

/// How often the trash is purged.
pub const EVERY: Duration = Duration::from_secs(60 * 60);

/// Deletes for good every page trashed more than 30 days ago, closing its room first. The only
/// path to permanent deletion besides a manager's explicit delete.
pub async fn purge(state: &AppState) -> Result<usize, sqlx::Error> {
    let ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM documents WHERE trashed_at < now() - interval '30 days'")
        .fetch_all(&state.pool)
        .await?;
    state.hub.close(&ids).await;
    let deleted = sqlx::query("DELETE FROM documents WHERE id = ANY($1) AND trashed_at < now() - interval '30 days'")
        .bind(&ids)
        .execute(&state.pool)
        .await?;
    Ok(deleted.rows_affected() as usize)
}

/// Purges every hour until `token` is cancelled. Run it in one replica.
pub async fn run(state: AppState, token: CancellationToken) {
    let mut tick = tokio::time::interval(EVERY);
    loop {
        tokio::select! {
            () = token.cancelled() => return,
            _ = tick.tick() => match purge(&state).await {
                Ok(0) => {}
                Ok(count) => tracing::info!(count, "documents: purged the trash"),
                Err(error) => tracing::error!(%error, "documents: trash purge failed"),
            },
        }
    }
}
