use axum::Router;
use http::request::Parts;
use porte::mcpauth::require_own_session;
use serde::Serialize;
use tronc::mcp::{Confirmations, PendingItem, PendingPage};

use crate::state::AppState;

#[derive(Serialize)]
struct Item {
    #[serde(flatten)]
    item: PendingItem,
    target: String,
}

/// `GET /mcp/pending` and `POST /mcp/pending/{id}/confirm|dismiss`, behind `require_auth`. Only
/// the person's own browser session passes: a bearer, an API token or a CLI token sent as a
/// cookie gets 403. Each action carries its `target`, which names the page.
pub fn routes(confirmations: Option<Confirmations>) -> Router<AppState> {
    PendingPage::new(confirmations, "docs-web", |parts: Parts| async move {
        require_own_session(&parts).map(|identity| identity.user_id.to_string())
    })
    .describe(|item, action| async move { Item { item, target: action.target } })
    .routes()
}
