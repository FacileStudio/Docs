//! `/mcp`: twelve curated tools over the documents, `docs_replace` behind a confirmation.

pub mod format;
mod manage;
mod page;
mod read;
mod replace;
mod tools;
mod write;

use std::collections::HashMap;

use tronc::mcp::{App, AppConfig, AppSpec, Call, ToolError};

use crate::state::AppState;

pub use replace::KIND;

/// The porte user id of the caller.
pub fn user(call: &Call) -> Result<i64, ToolError> {
    call.caller.user_id.parse().map_err(|_| ToolError("the token names no local account".to_owned()))
}

/// Docs' MCP server from `cfg`: the twelve tools, and `docs_replace` confirmed on the client's
/// `/pending` page. `cfg.pending` must be set.
pub fn app(state: &AppState, cfg: AppConfig) -> Result<App, String> {
    let spec = AppSpec {
        name: "docs".to_owned(),
        title: "Docs".to_owned(),
        instructions: format!(
            "Docs holds the team's pages, a tree per space. Read before you write: docs_read_blocks gives block ids \
             for docs_edit.\n\n{}",
            format::SHORT_FORM
        ),
        kinds: HashMap::from([(KIND.to_owned(), replace::kind(state))]),
        ..AppSpec::default()
    };
    let mut app = App::new(cfg, spec)?;
    let confirmations = app.confirmations.clone().ok_or("docs: docs_replace needs confirmations")?;
    app.options.tools = tools::all(state, &confirmations);
    Ok(app)
}
