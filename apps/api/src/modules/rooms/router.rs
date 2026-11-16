use axum::Router;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::ws::rejection::WebSocketUpgradeRejection;
use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::get;
use http::request::Parts;
use porte::Identity;
use salon::{Caller, Who};

use crate::extract::id;
use crate::modules::documents::access::{self, Access};
use crate::state::AppState;

async fn upgrade(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    parts: Parts,
    upgrade: Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
) -> tronc::Result<Response> {
    state.sessions.check_origin(&parts)?;
    let document = id(&raw)?;
    let have = access::check(&state, who.user_id, document, Access::Viewer).await?;
    let member = Who { user_id: Some(who.user_id), share: None, name: None, writer: have >= Access::Editor };
    Ok(salon::accept(state, document, Caller { who: member, session_id: who.session_id }, upgrade))
}

/// `GET /documents/{id}/ws`, behind `require_auth`: the y-sync WebSocket of one page.
pub fn routes() -> Router<AppState> {
    Router::new().route("/documents/{id}/ws", get(upgrade))
}
