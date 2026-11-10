use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tronc::mcp::{Answer, Call, ToolError};

use super::user;
use crate::extract::id;
use crate::modules::documents::types::CreateRequest;
use crate::modules::documents::{ops, service};
use crate::state::AppState;

/// `docs_create`.
#[derive(Deserialize, JsonSchema)]
pub struct CreateIn {
    /// The page title.
    pub title: String,
    /// The body, in Facile Markdown.
    pub markdown: String,
    /// The space; a personal page when absent and no parent is given.
    pub space_id: Option<String>,
    /// The parent page; the new page goes last among its children.
    pub parent_id: Option<String>,
}

/// What a write answers.
#[derive(Serialize, JsonSchema)]
pub struct Written {
    pub id: String,
    pub version: i64,
    pub markdown: String,
}

/// `docs_create`: a new page from markdown.
pub async fn create(state: AppState, call: Call, input: CreateIn) -> Result<Answer<Written>, ToolError> {
    let parent_id = input.parent_id.as_deref().map(id).transpose()?;
    let request = CreateRequest {
        title: input.title,
        space_id: input.space_id,
        parent_id,
        markdown: Some(input.markdown),
        content: None,
    };
    let doc = service::create(&state, user(&call)?, request).await?;
    let markdown = match &doc.body {
        crate::modules::documents::types::Body::Content(json) => serde_json::from_value::<encre::Node>(json.clone())
            .map(|n| encre::markdown::serialize(&n))
            .unwrap_or_default(),
        crate::modules::documents::types::Body::Markdown(text) => text.clone(),
    };
    Ok(Written { id: doc.id.to_string(), version: doc.version, markdown }.into())
}

/// `docs_edit`.
#[derive(Deserialize, JsonSchema)]
pub struct EditIn {
    /// The page id.
    pub document_id: String,
    /// Operations applied in order, all or nothing: `{"op": "insert_after", "block_id": …, "markdown": …}`.
    /// `replace_all` is not accepted here; use `docs_replace`.
    pub ops: Vec<Value>,
}

/// The operations of `input`, refusing `replace_all`.
pub fn parse_ops(raw: Vec<Value>) -> Result<Vec<encre::ops::Op>, ToolError> {
    let list: Vec<encre::ops::Op> = raw
        .into_iter()
        .map(|value| serde_json::from_value(value).map_err(|e| ToolError(format!("invalid operation: {e}"))))
        .collect::<Result<_, _>>()?;
    match list.iter().any(|op| matches!(op, encre::ops::Op::ReplaceAll { .. })) {
        true => {
            Err(ToolError("replace_all overwrites the page: call docs_replace, which asks the person first".into()))
        }
        false => Ok(list),
    }
}

/// `docs_edit`: guarded operations, applied live in open editors.
pub async fn edit(state: AppState, call: Call, input: EditIn) -> Result<Answer<Written>, ToolError> {
    let document = id(&input.document_id)?;
    let list = parse_ops(input.ops)?;
    let answer = ops::run(&state, user(&call)?, document, &list).await?;
    Ok(Written { id: input.document_id, version: answer.version, markdown: answer.markdown }.into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn replace_all_is_refused_by_docs_edit() {
        let ops = vec![serde_json::json!({"op": "replace_all", "markdown": "x", "expect_version": 1})];
        assert!(super::parse_ops(ops).unwrap_err().0.contains("docs_replace"));
        assert!(super::parse_ops(vec![serde_json::json!({"op": "append", "markdown": "x"})]).is_ok());
    }
}
