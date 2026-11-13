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

use super::service::{self, Upload, Uploaded};
use super::sniff::{self, MAX_BYTES};
use crate::extract::{Params, id};
use crate::state::AppState;

struct Part {
    name: String,
    declared: Option<String>,
    bytes: Vec<u8>,
}

async fn file_field(mut multipart: Multipart) -> Result<Part> {
    let invalid = |error: axum::extract::multipart::MultipartError| Error::invalid(error.body_text());
    while let Some(field) = multipart.next_field().await.map_err(invalid)? {
        if field.name() == Some("file") {
            let name = field.file_name().unwrap_or("file").to_owned();
            let declared = field.content_type().map(str::to_owned);
            return Ok(Part { name, declared, bytes: field.bytes().await.map_err(invalid)?.to_vec() });
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
    let part = file_field(multipart).await?;
    let file = Upload { name: &part.name, declared: part.declared.as_deref(), bytes: &part.bytes };
    Ok((StatusCode::CREATED, Json(service::upload(&state, who.user_id, document, file).await?)))
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
    let disposition = match sniff::inline(&attachment.mime) {
        true => format!("inline; filename=\"{}\"", attachment.name),
        false => download(&attachment.name),
    };
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

/// The `Content-Disposition` of a download named `name`, as RFC 6266 and RFC 8187 spell a
/// UTF-8 name.
fn download(name: &str) -> String {
    let attr = |b: u8| b.is_ascii_alphanumeric() || b"!#$&+-.^_`|~".contains(&b);
    let encoded: String =
        name.bytes().map(|b| if attr(b) { char::from(b).to_string() } else { format!("%{b:02X}") }).collect();
    format!("attachment; filename*=UTF-8''{encoded}")
}

/// Upload behind `require_auth`, with a body cap 1 MiB over the larger of the image cap and
/// `file_max`, for the multipart framing.
pub fn routes(sessions: &Manager, file_max: usize) -> Router<AppState> {
    let limit = DefaultBodyLimit::max(MAX_BYTES.max(file_max) + (1 << 20));
    let upload = post(upload).layer(limit).route_layer(sessions.require_auth());
    Router::new()
        .route("/documents/{id}/attachments", upload)
        .route("/attachments/{id}", get(serve).route_layer(sessions.optional()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn download_names_are_percent_encoded_utf8() {
        assert_eq!(super::download("Café 1.pdf"), "attachment; filename*=UTF-8''Caf%C3%A9%201.pdf");
        assert_eq!(super::download("a'b;c.zip"), "attachment; filename*=UTF-8''a%27b%3Bc.zip");
    }
}
