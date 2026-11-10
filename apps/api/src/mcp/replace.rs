use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Action, Answer, Call, Confirmations, Kind, ToolError};
use uuid::Uuid;

use super::user;
use crate::extract::id;
use crate::modules::documents::access::{self, Access};
use crate::modules::documents::{markdown, ops, service};
use crate::state::AppState;

/// The confirmation kind of `docs_replace`.
pub const KIND: &str = "docs_replace";

/// `docs_replace`, and what a pending action stores as its target.
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ReplaceIn {
    /// The page id.
    pub document_id: String,
    /// The new body, in Facile Markdown.
    pub markdown: String,
    /// The version `docs_read` or `docs_read_blocks` answered.
    pub expect_version: u64,
}

/// What `docs_replace` answers.
#[derive(Serialize, JsonSchema)]
pub struct ReplaceOut {
    /// `done`, `declined` or `pending_confirmation`.
    pub status: String,
    /// The link the person opens to confirm, when the client could not ask.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub confirm_url: String,
}

fn target(action: &Action) -> tronc::Result<(i64, Uuid, ReplaceIn)> {
    let input: ReplaceIn =
        serde_json::from_str(&action.target).map_err(|e| tronc::Error::internal("bad action target", e))?;
    let user = action.user_id.parse().map_err(|e| tronc::Error::internal("bad action user", e))?;
    Ok((user, id(&input.document_id)?, input))
}

async fn fingerprint(state: AppState, action: Action) -> tronc::Result<String> {
    let (user, document, _) = target(&action)?;
    access::check(&state, user, document, Access::Editor).await?;
    Ok(service::fetch(&state, document).await?.version.to_string())
}

async fn run(state: AppState, action: Action) -> tronc::Result<()> {
    let (user, document, input) = target(&action)?;
    let op = encre::ops::Op::ReplaceAll { markdown: input.markdown, expect_version: input.expect_version };
    ops::run(&state, user, document, &[op]).await.map(|_| ())
}

/// The confirmation kind that overwrites a page once the person agrees.
pub fn kind(state: &AppState) -> Kind {
    let (a, b) = (state.clone(), state.clone());
    Kind::new(KIND, move |action| fingerprint(a.clone(), action), move |action| run(b.clone(), action))
}

fn out(status: &str, confirm_url: String) -> ReplaceOut {
    ReplaceOut { status: status.to_owned(), confirm_url }
}

/// `docs_replace`: asks the person, then overwrites the whole page if they agree and the page
/// is still at `expect_version`.
pub async fn replace(
    state: AppState,
    confirmations: Confirmations,
    call: Call,
    input: ReplaceIn,
) -> Result<Answer<ReplaceOut>, ToolError> {
    if call.retrying() {
        let outcome = confirmations.resume(&call, KIND).await?;
        return Ok(out(outcome.status, outcome.confirm_url).into());
    }
    markdown::within(input.markdown.len(), state.markdown_max)?;
    let document = id(&input.document_id)?;
    access::check(&state, user(&call)?, document, Access::Editor).await?;
    let row = service::fetch(&state, document).await?;
    if row.version as u64 != input.expect_version {
        return Err(ToolError(format!(
            "the page is at version {}, not {}; read it again and retry",
            row.version, input.expect_version
        )));
    }
    let question = format!("Replace the whole content of \"{}\" with the agent's version?", row.title);
    let target = serde_json::to_string(&input).map_err(|e| ToolError(e.to_string()))?;
    let (ask, outcome) = confirmations.request(&call, KIND, &target, &question).await?;
    match ask {
        Some(question) => Ok(Answer::InputRequired(question)),
        None => Ok(out(outcome.status, outcome.confirm_url).into()),
    }
}
