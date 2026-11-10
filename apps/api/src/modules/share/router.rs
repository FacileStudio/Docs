use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use porte::Identity;
use serde::Deserialize;
use tronc::Result;
use tronc::http::{RateLimitConfig, RateLimiter, rate_limit};
use tronc::json::Json;

use super::public::{self, PublicPage};
use super::service::{self, Created, Link};
use crate::extract::id;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShareRequest {
    #[serde(default)]
    children: bool,
}

async fn create(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Json(body): Json<ShareRequest>,
) -> Result<(StatusCode, Json<Created>)> {
    Ok((StatusCode::CREATED, Json(service::create(&state, who.user_id, id(&raw)?, body.children).await?)))
}

async fn list(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Vec<Link>>> {
    service::list(&state, who.user_id, id(&raw)?).await.map(Json)
}

async fn revoke(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<StatusCode> {
    service::revoke(&state, who.user_id, id(&raw)?).await.map(|()| StatusCode::NO_CONTENT)
}

async fn root(State(state): State<AppState>, Path(token): Path<String>) -> Result<Json<PublicPage>> {
    public::root(&state, &token).await.map(Json)
}

async fn child(State(state): State<AppState>, Path((token, raw)): Path<(String, String)>) -> Result<Json<PublicPage>> {
    public::child(&state, &token, id(&raw)?).await.map(Json)
}

/// The link routes, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new().route("/documents/{id}/share", post(create).get(list)).route("/share/{id}", delete(revoke))
}

/// The public routes: no session, 2 requests a second per client IP with a burst of 30.
pub fn public_routes() -> Router<AppState> {
    let limiter = Arc::new(RateLimiter::new(RateLimitConfig { limit: 2.0, burst: 30, ..RateLimitConfig::default() }));
    Router::new()
        .route("/public/{token}", get(root))
        .route("/public/{token}/documents/{id}", get(child))
        .route_layer(axum::middleware::from_fn_with_state(limiter, rate_limit))
}
