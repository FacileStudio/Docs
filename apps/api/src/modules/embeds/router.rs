use axum::Router;
use axum::extract::State;
use axum::routing::get;
use porte::Identity;
use serde::Deserialize;
use tronc::json::Json;
use tronc::{Error, Result};
use url::Url;

use super::guard::allowed;
use super::{Preview, unfurl};
use crate::extract::Params;
use crate::state::AppState;

#[derive(Deserialize)]
struct ResolveQuery {
    url: String,
}

/// `raw` as a link an unfurl may fetch: `https`, at most 2048 characters, no credentials, and a
/// public address when it is an IP literal.
pub fn parse(raw: &str) -> Result<Url> {
    let url = Url::parse(raw.trim()).map_err(|_| Error::invalid("url is not a valid link"))?;
    match raw.len() <= 2048 && allowed(&url) {
        true => Ok(url),
        false => Err(Error::invalid("url must be a public https link of at most 2048 characters, without credentials")),
    }
}

async fn resolve(
    State(state): State<AppState>,
    _who: Identity,
    Params(query): Params<ResolveQuery>,
) -> Result<Json<Preview>> {
    let url = parse(&query.url)?;
    if let Some(preview) = state.previews.get(url.as_str()) {
        return Ok(Json(preview));
    }
    let preview = unfurl::unfurl(&state.previews.client, &url).await;
    state.previews.put(url.as_str(), preview.clone());
    Ok(Json(preview))
}

/// `GET /embeds/resolve?url=`, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new().route("/embeds/resolve", get(resolve))
}
