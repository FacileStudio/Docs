use axum::Router;
use axum::routing::get;
use tronc::health::{self, Check};

use crate::auth::{self, Kits};
use crate::modules::{assistant, attachments, documents, embeds, export, pending, rooms, search, share, spaces};
use crate::state::AppState;

async fn api_not_found() -> tronc::Error {
    tronc::Error::not_found("route not found")
}

fn protected(kits: &Kits, state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(spaces::routes())
        .merge(documents::routes())
        .merge(search::routes())
        .merge(share::routes())
        .merge(rooms::routes())
        .merge(export::routes())
        .merge(pending::routes(state.confirmations.clone()))
        .merge(embeds::routes())
        .merge(assistant::routes())
        .route_layer(kits.sessions.require_auth())
}

fn api(kits: &Kits, state: &AppState) -> Router<AppState> {
    let mut api = Router::new()
        .merge(auth::routes(kits))
        .merge(protected(kits, state))
        .merge(attachments::routes(&kits.sessions, state.attachment_max))
        .merge(share::public_routes());
    if let Some(syncer) = &kits.syncer {
        api = api.route("/registre/sync", syncer.nudge_route());
    }
    if let Some(deletions) = &kits.deletions {
        api = api.route("/registre/deletions", deletions.route());
    }
    api.fallback(api_not_found)
}

/// The whole HTTP tree, before tronc's layers: probes, `/version`, `/docs`, `/mcp` when
/// `mcp` is given, `/api`, then the SPA.
pub fn build_router(state: AppState, kits: &Kits, mcp: Option<tronc::mcp::App>) -> Result<Router, String> {
    let pool = state.pool.clone();
    let mut root = health::routes(vec![Check::db(pool)])
        .route("/version", get(crate::version::handler))
        .nest("/api", api(kits, &state))
        .with_state(state.clone())
        .merge(tronc::apiref::routes(&crate::apiref::config(), crate::apiref::document())?);
    if let Some(app) = mcp {
        root = app.mount(root)?;
    }
    let dir = tronc::spa::dir_from_env();
    Ok(match tronc::spa::available(&dir) {
        true => root.fallback_service(tronc::spa::handler(tronc::spa::Config { dir, ..Default::default() })),
        false => root.fallback(tronc::http::not_found),
    })
}
