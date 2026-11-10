use encre::ops::{Op, OpError};
use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access};
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

/// `POST /documents/{id}/ops`, and every MCP edit: the operations run in the document's room
/// as one Yjs transaction, so open editors see them live, and are persisted before the answer.
pub async fn run(state: &AppState, user_id: i64, id: Uuid, list: &[Op]) -> Result<OpsAnswer> {
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
}
