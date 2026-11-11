use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// The lowercase hex SHA-256 of `bytes`.
pub fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// `DATA_DIR/attachments/<first two hex>/<sha256>`.
pub fn path(data_dir: &Path, sha256: &str) -> PathBuf {
    data_dir.join("attachments").join(&sha256[..2]).join(sha256)
}

/// Marks `target` as just written, so the sweep's grace period starts again.
async fn refresh(target: PathBuf) -> io::Result<()> {
    let touch = move || std::fs::File::options().append(true).open(target)?.set_modified(std::time::SystemTime::now());
    tokio::task::spawn_blocking(touch).await.map_err(io::Error::other)?
}

/// Writes `bytes` under their digest, once: an identical file is already there, and only its
/// modification time is refreshed so the sweep cannot take it while its new row commits. The
/// write goes to a temporary name first, so a reader never sees half a file.
pub async fn store(data_dir: &Path, bytes: &[u8]) -> io::Result<String> {
    let sha256 = digest(bytes);
    let target = path(data_dir, &sha256);
    if tokio::fs::try_exists(&target).await? {
        refresh(target).await?;
        return Ok(sha256);
    }
    let dir = target.parent().ok_or_else(|| io::Error::other("attachment path has no parent"))?;
    tokio::fs::create_dir_all(dir).await?;
    let temporary = dir.join(format!(".{sha256}.{}", uuid::Uuid::new_v4()));
    tokio::fs::write(&temporary, bytes).await?;
    tokio::fs::rename(&temporary, &target).await?;
    Ok(sha256)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_same_bytes_are_stored_once_under_their_digest() {
        let dir = std::env::temp_dir().join(format!("docs-files-{}", uuid::Uuid::new_v4()));
        let a = store(&dir, b"hello").await.unwrap();
        let b = store(&dir, b"hello").await.unwrap();
        assert_eq!(a, b);
        assert_eq!(a, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
        assert_eq!(tokio::fs::read(path(&dir, &a)).await.unwrap(), b"hello");
        assert_eq!(std::fs::read_dir(path(&dir, &a).parent().unwrap()).unwrap().count(), 1);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
