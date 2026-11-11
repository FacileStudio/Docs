use axum::Router;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::ws::rejection::WebSocketUpgradeRejection;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use http::request::Parts;
use porte::Identity;

use super::ws::{Caller, MAX_FRAME, serve};
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
    let editor = have >= Access::Editor;
    let upgrade = match upgrade {
        Ok(upgrade) => upgrade,
        Err(rejection) => return Ok(rejection.into_response()),
    };
    let upgrade = upgrade.max_message_size(MAX_FRAME + (1 << 20)).max_frame_size(MAX_FRAME + (1 << 20));
    let caller = Caller { user_id: who.user_id, session_id: who.session_id, editor };
    Ok(upgrade.on_upgrade(move |socket| serve(socket, state, document, caller)))
}

/// `GET /documents/{id}/ws`, behind `require_auth`: the y-sync WebSocket of one page.
pub fn routes() -> Router<AppState> {
    Router::new().route("/documents/{id}/ws", get(upgrade))
}
