use encre::Node;
use serde_json::{Map, Value};
use tronc::{Error, Result};

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

/// The document a create asks for: Facile Markdown, encre JSON, or neither for an empty page.
pub fn from_input(markdown: Option<&str>, content: Option<Value>) -> Result<Node> {
    match (markdown, content) {
        (Some(_), Some(_)) => Err(Error::invalid("send markdown or content, not both")),
        (Some(markdown), None) => from_markdown(markdown),
        (None, Some(content)) => serde_json::from_value(content)
            .map_err(|error| Error::invalid(format!("content is not a document: {error}"))),
        (None, None) => Ok(empty()),
    }
}

/// Parses Facile Markdown; blank markdown is an empty page.
pub fn from_markdown(markdown: &str) -> Result<Node> {
    let node = encre::markdown::parse(markdown).map_err(|error| Error::invalid(format!("markdown: {error}")))?;
    Ok(if node.content.is_empty() { empty() } else { node })
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

    #[test]
    fn blank_markdown_is_one_empty_paragraph() {
        let prepared = prepare(from_markdown("  \n").unwrap()).unwrap();
        assert_eq!(prepared.node.content.len(), 1);
        assert_eq!(prepared.node.content[0].kind, "paragraph");
        assert!(prepared.node.content[0].id().is_some());
    }

    #[test]
    fn invalid_or_double_input_is_refused() {
        let content = serde_json::json!({"type": "doc", "content": [{"type": "heading", "attrs": {"level": 9}}]});
        let error = prepare(from_input(None, Some(content)).unwrap()).unwrap_err();
        assert_eq!(error.code, tronc::Code::InvalidArgument);
        assert!(from_input(Some("a"), Some(Value::Null)).is_err());
        let prepared = prepare(from_markdown("# Title\n\nSome **bold** text.").unwrap()).unwrap();
        assert_eq!(encre::markdown::serialize(&prepared.node), "# Title\n\nSome **bold** text.");
        assert_eq!(load(&prepared.json, 1).unwrap(), prepared.node);
    }
}
