use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, RESULT_BUDGET, ToolError, clip};

use super::user;
use crate::extract::id;
use crate::modules::documents::access::{self, Access};
use crate::modules::documents::types::ScopeQuery;
use crate::modules::documents::{content, service, tree};
use crate::modules::search;
use crate::state::AppState;

/// `docs_search`.
#[derive(Deserialize, JsonSchema)]
pub struct SearchIn {
    /// Words to find; quotes, `or` and `-word` work as in a web search.
    pub query: String,
    /// The space to search; personal pages when absent.
    pub space_id: Option<String>,
}

/// One page found.
#[derive(Serialize, JsonSchema)]
pub struct Found {
    pub id: String,
    pub title: String,
    pub snippet: String,
}

/// What `docs_search` answers.
#[derive(Serialize, JsonSchema)]
pub struct SearchOut {
    pub results: Vec<Found>,
}

/// `docs_search`: full-text search inside one scope.
pub async fn search(state: AppState, call: Call, input: SearchIn) -> Result<Answer<SearchOut>, ToolError> {
    let hits = search::search(&state, user(&call)?, &input.query, input.space_id.as_deref()).await?;
    let results =
        hits.into_iter().map(|h| Found { id: h.id.to_string(), title: h.title, snippet: h.snippet }).collect();
    Ok(SearchOut { results }.into())
}

/// `docs_list_tree`.
#[derive(Deserialize, JsonSchema)]
pub struct TreeIn {
    /// The space; personal pages when absent.
    pub space_id: Option<String>,
}

/// One page of the tree.
#[derive(Serialize, JsonSchema)]
pub struct Page {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
}

/// What `docs_list_tree` answers, parents before children, siblings in order.
#[derive(Serialize, JsonSchema)]
pub struct TreeOut {
    pub pages: Vec<Page>,
}

/// `docs_list_tree`: every live page of a scope.
pub async fn list_tree(state: AppState, call: Call, input: TreeIn) -> Result<Answer<TreeOut>, ToolError> {
    let query = ScopeQuery { space_id: input.space_id, trashed: false };
    let entries = tree::list(&state, user(&call)?, &query).await?;
    let pages = entries
        .into_iter()
        .map(|e| Page { id: e.id.to_string(), parent_id: e.parent_id.map(|p| p.to_string()), title: e.title })
        .collect();
    Ok(TreeOut { pages }.into())
}

/// `docs_read` and `docs_read_blocks`.
#[derive(Deserialize, JsonSchema)]
pub struct ReadIn {
    /// The page id.
    pub document_id: String,
    /// `docs_read_blocks` only: start and end (exclusive) top-level block index.
    pub range: Option<[usize; 2]>,
}

/// What `docs_read` answers.
#[derive(Serialize, JsonSchema)]
pub struct ReadOut {
    pub id: String,
    pub title: String,
    pub version: i64,
    pub markdown: String,
    pub truncated: bool,
}

async fn load(state: &AppState, call: &Call, raw: &str) -> Result<(encre::Node, String, i64), ToolError> {
    let document = id(raw)?;
    access::check(state, user(call)?, document, Access::Viewer).await?;
    let row = service::fetch(state, document).await?;
    Ok((content::load(&row.content.0, row.schema_version)?, row.title, row.version))
}

/// `docs_read`: the page as Facile Markdown, clipped to the result budget.
pub async fn read(state: AppState, call: Call, input: ReadIn) -> Result<Answer<ReadOut>, ToolError> {
    let (doc, title, version) = load(&state, &call, &input.document_id).await?;
    let markdown = encre::markdown::serialize(&doc);
    let (text, truncated) = clip(&markdown, RESULT_BUDGET);
    Ok(ReadOut { id: input.document_id, title, version, markdown: text.to_owned(), truncated }.into())
}

/// One top-level block.
#[derive(Serialize, JsonSchema)]
pub struct Block {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    pub markdown: String,
}

/// What `docs_read_blocks` answers.
#[derive(Serialize, JsonSchema)]
pub struct BlocksOut {
    pub version: i64,
    pub blocks: Vec<Block>,
}

/// `docs_read_blocks`: top-level blocks with their ids, for targeted edits.
pub async fn read_blocks(state: AppState, call: Call, input: ReadIn) -> Result<Answer<BlocksOut>, ToolError> {
    let (mut doc, _, version) = load(&state, &call, &input.document_id).await?;
    let op = encre::ops::Op::ReadBlocks { range: input.range };
    let outcome = encre::ops::apply(&mut doc, version as u64, &op).map_err(|e| ToolError(e.to_string()))?;
    let encre::ops::Outcome::Blocks(list) = outcome else {
        return Err(ToolError("read_blocks answered no blocks".into()));
    };
    let blocks = list.into_iter().map(|b| Block { id: b.id, kind: b.kind, markdown: b.markdown }).collect();
    Ok(BlocksOut { version, blocks }.into())
}
