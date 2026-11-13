use axum::body::Body;
use http::{Request, header};

use super::http::{Answer, raw};
use super::{App, Person};

pub const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01\x08\x06\0\0\0\x1f\x15\xc4\x89";

pub async fn upload(app: &App, who: &Person, doc: &str, name: &str, bytes: &[u8]) -> Answer {
    upload_as(app, who, doc, (name, "image/svg+xml"), bytes).await
}

pub async fn upload_as(app: &App, who: &Person, doc: &str, (name, mime): (&str, &str), bytes: &[u8]) -> Answer {
    let boundary = "docs-test-boundary";
    let mut body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\n\
         Content-Type: {mime}\r\n\r\n"
    )
    .into_bytes();
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    let request = Request::post(format!("/api/documents/{doc}/attachments"))
        .header(header::HOST, "docs.test")
        .header(header::AUTHORIZATION, format!("Bearer {}", who.token))
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(body))
        .unwrap();
    raw(&app.router, request).await
}
