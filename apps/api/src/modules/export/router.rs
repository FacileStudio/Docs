use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, header};
use axum::response::Response;
use axum::routing::get;
use porte::Identity;
use tronc::Result;

use super::pdf;
use crate::extract::id;
use crate::modules::documents::access::{self, Access};
use crate::modules::documents::{content, service};
use crate::state::AppState;

fn file(bytes: Vec<u8>, mime: &'static str, title: &str, extension: &str) -> Response {
    let name: String = title.chars().filter(|c| c.is_alphanumeric() || " -_".contains(*c)).collect();
    let name = if name.trim().is_empty() { "document".to_owned() } else { name.trim().to_owned() };
    let mut response = Response::new(Body::from(bytes));
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(mime));
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{name}.{extension}\"")) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    response
}

async fn load(state: &AppState, who: &Identity, raw: &str) -> Result<(encre::Node, String)> {
    let document = id(raw)?;
    access::check(state, who.user_id, document, Access::Viewer).await?;
    let row = service::fetch(state, document).await?;
    Ok((content::load(&row.content.0, row.schema_version)?, row.title))
}

async fn markdown(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Response> {
    let (doc, title) = load(&state, &who, &raw).await?;
    let text = encre::markdown::serialize(&doc).into_bytes();
    Ok(file(text, "text/markdown; charset=utf-8", &title, "md"))
}

async fn commonmark(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Response> {
    let (doc, title) = load(&state, &who, &raw).await?;
    let text = encre::markdown::serialize_commonmark(&doc).into_bytes();
    Ok(file(text, "text/markdown; charset=utf-8", &title, "md"))
}

async fn portable(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Response> {
    let (doc, title) = load(&state, &who, &raw).await?;
    let bytes = pdf::export(&state, who.user_id, doc, title.clone()).await?;
    Ok(file(bytes, "application/pdf", &title, "pdf"))
}

/// The export routes, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/documents/{id}/export.md", get(markdown))
        .route("/documents/{id}/export.commonmark", get(commonmark))
        .route("/documents/{id}/export.pdf", get(portable))
}
