use axum::Router;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::routing::{get, post};
use porte::Identity;
use porte::session::Manager;
use serde::Deserialize;
use tronc::json::Json;
use tronc::{Error, Result};

use super::service::{self, Uploaded};
use super::sniff::MAX_BYTES;
use crate::extract::{Params, id};
use crate::state::AppState;

async fn file_field(mut multipart: Multipart) -> Result<(String, Vec<u8>)> {
    let invalid = |error: axum::extract::multipart::MultipartError| Error::invalid(error.body_text());
    while let Some(field) = multipart.next_field().await.map_err(invalid)? {
        if field.name() == Some("file") {
            let name = field.file_name().unwrap_or("file").to_owned();
            return Ok((name, field.bytes().await.map_err(invalid)?.to_vec()));
        }
    }
    Err(Error::invalid("the multipart body has no `file` part"))
}

async fn upload(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<Uploaded>)> {
    let document = id(&raw)?;
    let (name, bytes) = file_field(multipart).await?;
    Ok((StatusCode::CREATED, Json(service::upload(&state, who.user_id, document, &name, &bytes).await?)))
}

#[derive(Deserialize)]
struct ShareQuery {
    share: Option<String>,
}

async fn serve(
    State(state): State<AppState>,
    who: Option<Identity>,
    Path(raw): Path<String>,
    Params(query): Params<ShareQuery>,
) -> Result<Response> {
    let user = who.map(|who| who.user_id);
    let (attachment, bytes) = service::open(&state, user, query.share.as_deref(), id(&raw)?).await?;
    let disposition = format!("inline; filename=\"{}\"", attachment.name);
    let mut response = Response::new(Body::from(bytes));
    let headers = response.headers_mut();
    headers
        .insert(header::CONTENT_TYPE, HeaderValue::from_str(&attachment.mime).map_err(|e| Error::internal("mime", e))?);
    headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static("sandbox"));
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, max-age=3600"));
    if let Ok(value) = HeaderValue::from_str(&disposition) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    Ok(response)
}

/// Upload behind `require_auth`, with a body cap a little over 20 MiB for the multipart framing.
pub fn routes(sessions: &Manager) -> Router<AppState> {
    let upload = post(upload).layer(DefaultBodyLimit::max(MAX_BYTES + (1 << 20))).route_layer(sessions.require_auth());
    Router::new()
        .route("/documents/{id}/attachments", upload)
        .route("/attachments/{id}", get(serve).route_layer(sessions.optional()))
}
