use encre::ops::{Op, OpError};
use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access};
use super::markdown;
use super::types::OpsAnswer;
use crate::modules::rooms::Command;
use crate::state::AppState;

/// The most operations one request applies.
pub const MAX_OPS: usize = 100;

/// An operation refusal as the suite answers it: a stale `replace_all` is `412`, the rest `400`.
pub fn refusal(error: OpError) -> Error {
    match error {
        OpError::Version { .. } => Error::failed(error.to_string()),
        _ => Error::invalid(error.to_string()),
    }
}

/// How many bytes of markdown `list` asks encre to parse: a `patch` parses its replacement once
/// per expected match.
pub fn markdown_bytes(list: &[Op]) -> usize {
    list.iter()
        .map(|op| match op {
            Op::Append { markdown } | Op::Prepend { markdown } => markdown.len(),
            Op::InsertAfter { markdown, .. } | Op::ReplaceBlock { markdown, .. } => markdown.len(),
            Op::ReplaceAll { markdown, .. } => markdown.len(),
            Op::Patch { replace, expect, .. } => replace.len().saturating_mul(*expect),
            _ => 0,
        })
        .fold(0, usize::saturating_add)
}

/// `POST /documents/{id}/ops`, and every MCP edit: the operations run in the document's room
/// as one Yjs transaction, so open editors see them live, and are persisted before the answer.
/// Their markdown, all operations together, is capped at `state.markdown_max` bytes.
pub async fn run(state: &AppState, user_id: i64, id: Uuid, list: &[Op]) -> Result<OpsAnswer> {
    markdown::within(markdown_bytes(list), state.markdown_max)?;
    access::check(state, user_id, id, Access::Editor).await?;
    state.hub.ask(state, id, |reply| Command::Ops(list.to_vec(), user_id, reply)).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_stale_replace_all_is_a_precondition_failure() {
        let error = encre::ops::OpError::Version { expected: 1, actual: 2 };
        assert_eq!(super::refusal(error).code, tronc::Code::FailedPrecondition);
    }

    #[test]
    fn markdown_bytes_counts_every_parsed_field() {
        let list: Vec<encre::ops::Op> = serde_json::from_value(serde_json::json!([
            {"op": "append", "markdown": "abc"},
            {"op": "insert_after", "block_id": "b", "markdown": "de"},
            {"op": "patch", "find": "zzzz", "replace": "xy", "expect": 3},
            {"op": "delete_block", "block_id": "b"}
        ]))
        .unwrap();
        assert_eq!(super::markdown_bytes(&list), 3 + 2 + 6);
    }
}
