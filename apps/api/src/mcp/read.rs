use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, RESULT_BUDGET, ToolError, clip, encoded_size, fit_count};

use super::page::{Listed, Window, window};
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
    /// The `next_cursor` of the previous call; the first pages when absent.
    pub cursor: Option<String>,
    /// How many pages, 1 to 200; 50 when absent.
    pub limit: Option<usize>,
}

/// One page of the tree.
#[derive(Serialize, JsonSchema)]
pub struct Page {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
}

/// `docs_list_tree`: the live pages of a scope, parents before children, siblings in order, a
/// window at a time.
pub async fn list_tree(state: AppState, call: Call, input: TreeIn) -> Result<Answer<Listed<Page>>, ToolError> {
    let query = ScopeQuery { space_id: input.space_id, trashed: false };
    let entries = tree::list(&state, user(&call)?, &query).await?;
    let pages = entries
        .into_iter()
        .map(|e| Page { id: e.id.to_string(), parent_id: e.parent_id.map(|p| p.to_string()), title: e.title })
        .collect();
    Ok(window(pages, Window { cursor: input.cursor, limit: input.limit })?.into())
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
    /// The result budget cut the list, or the one block it holds; read on with `next_range`.
    pub truncated: bool,
    /// The `range` that reads the blocks this answer left out; absent when none were.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_range: Option<[usize; 2]>,
}

/// The blocks of `list`, read from top-level index `start`, cut to the result budget. A single
/// block past the budget comes back with its markdown clipped.
fn within_budget(version: i64, list: Vec<Block>, start: usize) -> BlocksOut {
    let end = start + list.len();
    let fit = fit_count(list.len(), |n| encoded_size(&list[..n]));
    let mut blocks = list;
    let clipped = fit == 0 && !blocks.is_empty();
    blocks.truncate(fit.max(1));
    if let Some(block) = blocks.first_mut().filter(|_| clipped) {
        block.markdown = clip(&block.markdown, RESULT_BUDGET / 2).0.to_owned();
    }
    let next = start + blocks.len();
    let next_range = (next < end).then_some([next, end]);
    BlocksOut { version, blocks, truncated: clipped || next_range.is_some(), next_range }
}

/// `docs_read_blocks`: top-level blocks with their ids, for targeted edits, within the result
/// budget.
pub async fn read_blocks(state: AppState, call: Call, input: ReadIn) -> Result<Answer<BlocksOut>, ToolError> {
    let (mut doc, _, version) = load(&state, &call, &input.document_id).await?;
    let op = encre::ops::Op::ReadBlocks { range: input.range };
    let outcome = encre::ops::apply(&mut doc, version as u64, &op).map_err(|e| ToolError(e.to_string()))?;
    let encre::ops::Outcome::Blocks(list) = outcome else {
        return Err(ToolError("read_blocks answered no blocks".into()));
    };
    let blocks = list.into_iter().map(|b| Block { id: b.id, kind: b.kind, markdown: b.markdown }).collect();
    let start = input.range.map_or(0, |[start, _]| start);
    Ok(within_budget(version, blocks, start).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocks(sizes: &[usize]) -> Vec<Block> {
        sizes
            .iter()
            .map(|&n| Block { id: Some(format!("b{n}")), kind: "paragraph".into(), markdown: "x".repeat(n) })
            .collect()
    }

    #[test]
    fn blocks_stop_at_the_result_budget_and_name_the_rest() {
        let small = within_budget(3, blocks(&[10, 20]), 0);
        assert_eq!((small.blocks.len(), small.truncated, small.next_range), (2, false, None));
        let cut = within_budget(3, blocks(&[30_000; 6]), 4);
        assert_eq!((cut.blocks.len(), cut.truncated, cut.next_range), (2, true, Some([6, 10])));
        assert!(encoded_size(&cut) <= RESULT_BUDGET + 100);
        let huge = within_budget(3, blocks(&[200_000, 5]), 0);
        assert_eq!((huge.blocks.len(), huge.truncated, huge.next_range), (1, true, Some([1, 2])));
        assert!(encoded_size(&huge) <= RESULT_BUDGET);
    }
}
