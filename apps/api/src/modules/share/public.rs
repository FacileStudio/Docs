use serde::Serialize;
use serde_json::Value;
use tronc::{Error, Result};
use uuid::Uuid;

use super::service::{self, Share};
use crate::extract::db;
use crate::modules::documents::service::fetch;
use crate::modules::documents::types::TreeEntry;
use crate::state::AppState;

/// What the public routes answer: one page, read-only, and the shared tree under the link.
#[derive(Debug, Serialize)]
pub struct PublicPage {
    pub id: Uuid,
    pub title: String,
    pub icon: String,
    pub content: Value,
    pub children: Vec<TreeEntry>,
}

/// Appends `?share=<token>` to every image whose `src` is one of this app's attachments, so
/// a reader without a session can load it.
pub fn share_images(node: &mut Value, token: &str) {
    if node["type"] == "image"
        && let Some(src) =
            node["attrs"]["src"].as_str().filter(|s| s.starts_with("/api/attachments/") && !s.contains('?'))
    {
        node["attrs"]["src"] = Value::String(format!("{src}?share={token}"));
    }
    if let Some(children) = node.get_mut("content").and_then(Value::as_array_mut) {
        children.iter_mut().for_each(|child| share_images(child, token));
    }
}

async fn subtree(state: &AppState, root: Uuid) -> Result<Vec<TreeEntry>> {
    sqlx::query_as::<_, TreeEntry>(
        "WITH RECURSIVE sub(id) AS (SELECT id FROM documents WHERE parent_id = $1 AND trashed_at IS NULL \
         UNION SELECT d.id FROM documents d JOIN sub ON d.parent_id = sub.id WHERE d.trashed_at IS NULL) \
         SELECT d.id, d.parent_id, d.position, d.title, d.icon, d.updated_at, d.trashed_at FROM documents d \
         JOIN sub ON sub.id = d.id ORDER BY d.parent_id, d.position, d.id",
    )
    .bind(root)
    .fetch_all(&state.pool)
    .await
    .map_err(db)
}

async fn page(state: &AppState, token: &str, id: Uuid, children: Vec<TreeEntry>) -> Result<PublicPage> {
    let row = fetch(state, id).await?;
    let mut content = row.content.0;
    share_images(&mut content, token);
    Ok(PublicPage { id, title: row.title, icon: row.icon, content, children })
}

/// `GET /public/{token}`: the shared page, and its tree when the link covers children.
pub async fn root(state: &AppState, token: &str) -> Result<PublicPage> {
    let share: Share = service::resolve(state, token).await?;
    let children = match share.children {
        true => subtree(state, share.document_id).await?,
        false => Vec::new(),
    };
    page(state, token, share.document_id, children).await
}

/// `GET /public/{token}/documents/{id}`: a page of the shared subtree.
pub async fn child(state: &AppState, token: &str, id: Uuid) -> Result<PublicPage> {
    let share = service::resolve(state, token).await?;
    if id == share.document_id || !service::covers(state, &share, id).await? {
        return Err(Error::not_found("document not found"));
    }
    page(state, token, id, Vec::new()).await
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn only_local_attachment_images_get_the_token() {
        let mut doc = json!({"type": "doc", "content": [
            {"type": "image", "attrs": {"src": "/api/attachments/abc"}},
            {"type": "image", "attrs": {"src": "https://example.test/a.png"}},
            {"type": "blockquote", "content": [{"type": "image", "attrs": {"src": "/api/attachments/def"}}]}
        ]});
        super::share_images(&mut doc, "tok");
        assert_eq!(doc["content"][0]["attrs"]["src"], "/api/attachments/abc?share=tok");
        assert_eq!(doc["content"][1]["attrs"]["src"], "https://example.test/a.png");
        assert_eq!(doc["content"][2]["content"][0]["attrs"]["src"], "/api/attachments/def?share=tok");
    }
}
