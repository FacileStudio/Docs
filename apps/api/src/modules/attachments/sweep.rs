use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::state::AppState;

/// How often unreferenced files are removed.
pub const EVERY: Duration = Duration::from_secs(24 * 60 * 60);

/// A file younger than this is kept: its row may not be committed yet.
pub const GRACE: Duration = Duration::from_secs(60 * 60);

type SweepError = Box<dyn std::error::Error + Send + Sync>;

async fn on_disk(root: &Path) -> std::io::Result<Vec<(String, std::path::PathBuf)>> {
    let mut found = Vec::new();
    let Ok(mut shards) = tokio::fs::read_dir(root).await else { return Ok(found) };
    while let Some(shard) = shards.next_entry().await? {
        let mut files = tokio::fs::read_dir(shard.path()).await?;
        while let Some(file) = files.next_entry().await? {
            let age = file.metadata().await?.modified()?.elapsed().unwrap_or_default();
            if age >= GRACE {
                found.push((file.file_name().to_string_lossy().into_owned(), file.path()));
            }
        }
    }
    Ok(found)
}

/// Removes every file older than [`GRACE`] that no `attachments` row names, and answers how
/// many went.
pub async fn sweep(state: &AppState) -> Result<usize, SweepError> {
    let files = on_disk(&state.data_dir.join("attachments")).await?;
    let kept: HashSet<String> = sqlx::query_scalar("SELECT DISTINCT sha256 FROM attachments")
        .fetch_all(&state.pool)
        .await?
        .into_iter()
        .collect();
    let mut removed = 0;
    for (name, path) in files.into_iter().filter(|(name, _)| !kept.contains(name)) {
        tokio::fs::remove_file(&path).await?;
        tracing::info!(file = %name, "attachments: removed an unreferenced file");
        removed += 1;
    }
    Ok(removed)
}

/// Sweeps once a day until `token` is cancelled. Run it in one replica.
pub async fn run(state: AppState, token: CancellationToken) {
    let mut tick = tokio::time::interval(EVERY);
    loop {
        tokio::select! {
            () = token.cancelled() => return,
            _ = tick.tick() => {
                if let Err(error) = sweep(&state).await {
                    tracing::error!(%error, "attachments: sweep failed");
                }
            }
        }
    }
}
