use std::path::Path;

use sqlx::types::Json;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use super::Args;
use super::client::Client;
use super::plan::{Page, Plan};
use crate::modules::attachments::{files, sniff};
use crate::modules::documents::content::{self, Prepared};
use crate::modules::documents::markdown;

type WriteError = Box<dyn std::error::Error + Send + Sync>;

struct Stored {
    id: Uuid,
    mime: &'static str,
    size: usize,
    sha256: String,
}

async fn fetch_attachments(client: &Client, data: &Path, page: &mut Page) -> Result<Vec<Stored>, WriteError> {
    let mut stored = Vec::new();
    for path in page.attachments.clone() {
        let bytes = client.attachment(&path).await?;
        let mime =
            sniff::sniff(&bytes).ok_or_else(|| format!("{}: attachment {path} is not an accepted type", page.title))?;
        let sha256 = files::store(data, &bytes).await?;
        let id = Uuid::new_v4();
        page.markdown = page.markdown.replace(&path, &format!("/api/attachments/{id}"));
        stored.push(Stored { id, mime, size: bytes.len(), sha256 });
    }
    Ok(stored)
}

async fn insert(tx: &mut PgConnection, space: &str, page: &Page, prepared: Prepared) -> Result<(), WriteError> {
    sqlx::query(
        "INSERT INTO documents (id, space_id, parent_id, position, title, icon, content, text, created_by, updated_by, \
         created_at, updated_at, schema_version) SELECT $1, $2, $3, $4, $5, $6, $7, $8, u.id, u.id, $10, $11, $12 \
         FROM (SELECT (SELECT id FROM porte_users WHERE lower(email) = lower($9)) AS id) u",
    )
    .bind(page.id)
    .bind(space)
    .bind(page.parent)
    .bind(page.position)
    .bind(&page.title)
    .bind(page.icon.chars().take(32).collect::<String>())
    .bind(Json(&prepared.json))
    .bind(&prepared.text)
    .bind(&page.author)
    .bind(page.created_at)
    .bind(page.updated_at)
    .bind(encre::SCHEMA_VERSION as i32)
    .execute(&mut *tx)
    .await?;
    Ok(())
}

async fn attachments(tx: &mut PgConnection, page: &Page, stored: &[Stored]) -> Result<(), WriteError> {
    for file in stored {
        sqlx::query(
            "INSERT INTO attachments (id, document_id, name, mime, size, sha256) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(file.id)
        .bind(page.id)
        .bind(format!("{}.{}", file.id, file.mime.rsplit('/').next().unwrap_or("bin")))
        .bind(file.mime)
        .bind(file.size as i64)
        .bind(&file.sha256)
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}

async fn import_page(tx: &mut PgConnection, args: &Args, page: &Page, stored: &[Stored]) -> Result<(), WriteError> {
    let node = markdown::parse(page.markdown.clone(), args.markdown_max).await;
    let prepared = node.and_then(content::prepare).map_err(|e| format!("{}: {}", page.title, e.message))?;
    insert(tx, &args.space, page, prepared).await?;
    attachments(tx, page, stored).await
}

/// Writes `plan` into the space of `args` in one transaction: all of it, or nothing.
pub async fn write(pool: &PgPool, client: &Client, data: &Path, args: &Args, plan: Plan) -> Result<usize, WriteError> {
    let space = args.space.as_str();
    let known: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM spaces WHERE id = $1)").bind(space).fetch_one(pool).await?;
    if !known {
        return Err(format!("space {space} is not in the directory yet: sign in once or wait for the sync").into());
    }
    let mut tx = pool.begin().await?;
    let count = plan.pages.len();
    for mut page in plan.pages {
        let stored = fetch_attachments(client, data, &mut page).await?;
        import_page(&mut tx, args, &page, &stored).await?;
    }
    tx.commit().await?;
    Ok(count)
}
