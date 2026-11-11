use std::collections::HashMap;

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use super::client::{Client, Collection, Document, NavNode};
use super::convert::convert;

type PlanError = Box<dyn std::error::Error + Send + Sync>;

/// One page to create.
#[derive(Debug, Clone)]
pub struct Page {
    pub id: Uuid,
    pub parent: Option<Uuid>,
    pub position: f64,
    pub title: String,
    pub icon: String,
    pub markdown: String,
    pub attachments: Vec<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub author: Option<String>,
}

/// Everything an import would write, parents before children, and what it could not convert.
#[derive(Debug, Default)]
pub struct Plan {
    pub pages: Vec<Page>,
    pub unconverted: Vec<(String, String)>,
}

/// An Outline icon kept only when it is an emoji: Outline also stores names of its own icon set
/// (`academicCap`, `collection`), which Docs cannot draw and would print as text.
fn emoji(icon: Option<&str>) -> String {
    let icon = icon.unwrap_or_default();
    if icon.chars().any(|c| c.is_ascii_alphabetic()) { String::new() } else { icon.to_owned() }
}

fn stamp(raw: &str) -> OffsetDateTime {
    OffsetDateTime::parse(raw, &Rfc3339).unwrap_or_else(|_| OffsetDateTime::now_utc())
}

struct Source {
    docs: HashMap<String, Document>,
    text: HashMap<String, String>,
    ids: HashMap<String, Uuid>,
}

fn walk(plan: &mut Plan, source: &Source, nodes: &[NavNode], parent: Uuid) {
    for (index, node) in nodes.iter().enumerate() {
        let (Some(doc), Some(id)) = (source.docs.get(&node.id), source.ids.get(&node.id)) else { continue };
        let done = convert(source.text.get(&node.id).map_or("", String::as_str), &source.ids);
        plan.unconverted.extend(done.unconverted.into_iter().map(|what| (doc.title.clone(), what)));
        plan.pages.push(Page {
            id: *id,
            parent: Some(parent),
            position: (index + 1) as f64,
            title: doc.title.clone(),
            icon: emoji(doc.icon.as_deref()),
            markdown: done.markdown,
            attachments: done.attachments,
            created_at: stamp(&doc.created_at),
            updated_at: stamp(&doc.updated_at),
            author: doc.created_by.as_ref().and_then(|a| a.email.clone()),
        });
        walk(plan, source, &node.children, *id);
    }
}

fn collection_page(collection: &Collection, position: usize) -> Page {
    let now = OffsetDateTime::now_utc();
    Page {
        id: Uuid::new_v4(),
        parent: None,
        position: position as f64,
        title: collection.name.clone(),
        icon: emoji(collection.icon.as_deref()),
        markdown: collection.description.clone().unwrap_or_default(),
        attachments: Vec::new(),
        created_at: now,
        updated_at: now,
        author: None,
    }
}

async fn fetch(client: &Client, collection: &Collection) -> Result<(Vec<NavNode>, Source), PlanError> {
    let tree = client.tree(&collection.id).await?;
    let documents = client.documents(&collection.id).await?;
    let mut text = HashMap::new();
    for doc in &documents {
        text.insert(doc.id.clone(), client.export(&doc.id).await?);
    }
    let ids = documents.iter().map(|d| (d.id.clone(), Uuid::new_v4())).collect();
    let docs = documents.into_iter().map(|d| (d.id.clone(), d)).collect();
    Ok((tree, Source { docs, text, ids }))
}

/// Reads every collection and document and builds the plan: a collection becomes a top-level
/// page, each document keeps its parent and its order.
pub async fn build(client: &Client) -> Result<Plan, PlanError> {
    let mut plan = Plan::default();
    let mut fetched = Vec::new();
    for (index, collection) in client.collections().await?.iter().enumerate() {
        let (tree, source) = fetch(client, collection).await?;
        fetched.push((collection_page(collection, index + 1), tree, source));
    }
    let by_url: HashMap<String, Uuid> = fetched
        .iter()
        .flat_map(|(_, _, s)| s.docs.values().filter_map(|d| s.ids.get(&d.id).map(|id| (d.url_id.clone(), *id))))
        .collect();
    for (root, tree, mut source) in fetched {
        let root_id = root.id;
        plan.pages.push(root);
        source.ids.extend(by_url.clone());
        walk(&mut plan, &source, &tree, root_id);
    }
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::emoji;

    #[test]
    fn only_emoji_icons_survive_the_import() {
        assert_eq!(emoji(Some("🚀")), "🚀");
        assert_eq!(emoji(Some("1️⃣")), "1️⃣");
        assert_eq!(emoji(Some("academicCap")), "");
        assert_eq!(emoji(Some("collection")), "");
        assert_eq!(emoji(None), "");
    }
}
