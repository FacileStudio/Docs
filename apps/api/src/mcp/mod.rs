//! `/mcp`: seven curated tools over the documents, `docs_replace` behind a confirmation.

pub mod format;
mod read;
mod replace;
mod tools;
mod write;

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use tronc::mcp::{Call, ConfirmConfig, Confirmations, Options, PostgresPending, ToolError};

use crate::state::AppState;

pub use replace::KIND;

/// The porte user id of the caller.
pub fn user(call: &Call) -> Result<i64, ToolError> {
    call.caller.user_id.parse().map_err(|_| ToolError("the token names no local account".to_owned()))
}

/// The origin of `resource`, `https://docs.facile.studio` for `https://docs.facile.studio/mcp`.
pub fn origin(resource: &str) -> &str {
    resource.strip_suffix("/mcp").unwrap_or(resource)
}

/// The confirmation flow: actions in `mcp_actions`, confirmed on the client's `/pending` page.
pub fn confirmations(state: &AppState, resource: &str, key: &[u8]) -> Result<Confirmations, String> {
    Confirmations::new(ConfirmConfig {
        store: Arc::new(PostgresPending::new(state.pool.clone())),
        key: key.to_vec(),
        confirm_url: format!("{}/pending", origin(resource)),
        kinds: HashMap::from([(KIND.to_owned(), replace::kind(state))]),
        pending_ttl: std::time::Duration::ZERO,
        ask_ttl: std::time::Duration::ZERO,
    })
}

/// Mounts `/mcp` and its metadata on `router`, outside `/api` and outside the CSRF rule.
pub fn mount(
    router: Router,
    state: &AppState,
    options: Options,
    confirmations: Confirmations,
) -> Result<Router, String> {
    let mut options = options;
    options.title = "Docs".to_owned();
    options.version = crate::version::VERSION.to_owned();
    options.instructions = format!(
        "Docs holds the team's pages, a tree per space. Read before you write: docs_read_blocks gives block ids for \
         docs_edit.\n\n{}",
        format::SHORT_FORM
    );
    options.tools = tools::all(state, &confirmations);
    tronc::mcp::mount(router, options)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_origin_drops_the_mcp_path() {
        assert_eq!(super::origin("https://docs.facile.studio/mcp"), "https://docs.facile.studio");
    }
}
