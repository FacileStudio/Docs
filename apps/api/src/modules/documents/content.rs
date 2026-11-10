use encre::Node;
use serde_json::{Map, Value};
use tronc::{Error, Result};

use super::markdown;

/// A document JSON larger than this refuses further writes.
pub const MAX_CONTENT_BYTES: usize = 5 << 20;

/// A validated document in its stored form, with the plain text search reads.
#[derive(Debug, Clone)]
pub struct Prepared {
    /// The compact tree.
    pub node: Node,
    /// The tree as JSON, for `documents.content`.
    pub json: Value,
    /// `encre::markdown::plain_text`, for `documents.text`.
    pub text: String,
}

/// A document holding one empty paragraph: `doc` needs at least one block.
pub fn empty() -> Node {
    Node::new("doc", Map::new(), vec![Node::new("paragraph", Map::new(), Vec::new())])
}

/// The document a create asks for: Facile Markdown of at most `max` bytes, encre JSON, or neither
/// for an empty page.
pub async fn from_input(markdown: Option<String>, content: Option<Value>, max: usize) -> Result<Node> {
    match (markdown, content) {
        (Some(_), Some(_)) => Err(Error::invalid("send markdown or content, not both")),
        (Some(markdown), None) => markdown::parse(markdown, max).await,
        (None, Some(content)) => serde_json::from_value(content)
            .map_err(|error| Error::invalid(format!("content is not a document: {error}"))),
        (None, None) => Ok(empty()),
    }
}

/// Gives every block an id, then [`measure`]s. Every API write goes through it.
pub fn prepare(mut node: Node) -> Result<Prepared> {
    encre::assign_ids(&mut node);
    measure(node)
}

/// Validates, compacts and measures `node`, keeping its ids as they are: a room stores what its
/// editors hold, ids included.
pub fn measure(node: Node) -> Result<Prepared> {
    encre::validate(&node).map_err(|error| Error::invalid(format!("invalid document: {error}")))?;
    let node = encre::compact(&node);
    let json = serde_json::to_value(&node).map_err(|error| Error::internal("failed to encode the document", error))?;
    if json.to_string().len() > MAX_CONTENT_BYTES {
        return Err(Error::too_large("the document is over 5 MiB"));
    }
    let text = encre::markdown::plain_text(&node);
    Ok(Prepared { node, json, text })
}

/// Reads a stored document, upgrading it from `schema_version` to the current schema.
pub fn load(content: &Value, schema_version: i32) -> Result<Node> {
    let node: Node = serde_json::from_value(content.clone())
        .map_err(|error| Error::internal("a stored document does not decode", error))?;
    let from = u32::try_from(schema_version).unwrap_or(0);
    encre::migrate(node, from).map_err(|error| Error::internal("a stored document does not migrate", error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::documents::markdown::from_markdown;

    #[test]
    fn blank_markdown_is_one_empty_paragraph() {
        let prepared = prepare(from_markdown("  \n").unwrap()).unwrap();
        assert_eq!(prepared.node.content.len(), 1);
        assert_eq!(prepared.node.content[0].kind, "paragraph");
        assert!(prepared.node.content[0].id().is_some());
    }

    #[tokio::test]
    async fn invalid_or_double_input_is_refused() {
        let content = serde_json::json!({"type": "doc", "content": [{"type": "heading", "attrs": {"level": 9}}]});
        let error = prepare(from_input(None, Some(content), 64).await.unwrap()).unwrap_err();
        assert_eq!(error.code, tronc::Code::InvalidArgument);
        assert!(from_input(Some("a".into()), Some(Value::Null), 64).await.is_err());
        let prepared = prepare(from_markdown("# Title\n\nSome **bold** text.").unwrap()).unwrap();
        assert_eq!(encre::markdown::serialize(&prepared.node), "# Title\n\nSome **bold** text.");
        assert_eq!(load(&prepared.json, 1).unwrap(), prepared.node);
    }
}
