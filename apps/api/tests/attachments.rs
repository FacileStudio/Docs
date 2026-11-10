mod common;

use common::fixture;
use common::upload::{PNG, upload};
use serde_json::json;

#[tokio::test]
async fn an_upload_is_typed_by_its_bytes_and_served_sandboxed() {
    let Some(pool) = common::pool("docs_attach").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let doc = fixture::id(&fixture::page(&app, &roles.member, Some(&roles.space), "pics").await);
    let answer = upload(&app, &roles.member, &doc, "../dot.png", PNG).await;
    assert_eq!(answer.status, 201, "{}", answer.text());
    let url = answer.json()["url"].as_str().unwrap().to_owned();
    let served = app.call("GET", &url, Some(&roles.admin), None).await;
    assert_eq!(served.status, 200);
    assert_eq!(served.bytes, PNG);
    assert_eq!(served.headers["content-type"], "image/png");
    assert_eq!(served.headers["content-security-policy"], "sandbox");
    assert_eq!(served.headers["x-content-type-options"], "nosniff");
    assert_eq!(served.headers["content-disposition"], "inline; filename=\"dot.png\"");
    assert_eq!(app.call("GET", &url, Some(&roles.outsider), None).await.status, 404);
    assert_eq!(app.call("GET", &url, None, None).await.status, 401);
    let again = upload(&app, &roles.member, &doc, "copy.png", PNG).await.json();
    assert_ne!(again["id"], answer.json()["id"]);
}

#[tokio::test]
async fn svg_and_unknown_bytes_are_refused_whatever_the_request_claims() {
    let Some(pool) = common::pool("docs_attach_refuse").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let doc = fixture::id(&fixture::page(&app, &roles.member, None, "x").await);
    let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script>alert(1)</script></svg>";
    assert_eq!(upload(&app, &roles.member, &doc, "a.svg", svg).await.status, 400);
    assert_eq!(upload(&app, &roles.member, &doc, "a.png", b"not a png").await.status, 400);
    assert_eq!(upload(&app, &roles.outsider, &doc, "a.png", PNG).await.status, 404);
    let big = [PNG, &vec![0u8; 20 << 20]].concat();
    assert_eq!(upload(&app, &roles.member, &doc, "big.png", &big).await.status, 413);
}

#[tokio::test]
async fn the_sweep_removes_only_old_files_no_row_names() {
    let Some(pool) = common::pool("docs_sweep").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let doc = fixture::id(&fixture::page(&app, &roles.member, None, "x").await);
    upload(&app, &roles.member, &doc, "kept.png", PNG).await;
    let orphan = api::modules::attachments::files::store(&app.state.data_dir, b"%PDF-orphan").await.unwrap();
    let young = api::modules::attachments::files::store(&app.state.data_dir, b"%PDF-young").await.unwrap();
    let old = std::time::SystemTime::now() - std::time::Duration::from_secs(2 * 60 * 60);
    for sha in [&orphan, &api::modules::attachments::files::digest(PNG)] {
        let path = api::modules::attachments::files::path(&app.state.data_dir, sha);
        std::fs::File::options().write(true).open(path).unwrap().set_modified(old).unwrap();
    }
    assert_eq!(api::modules::attachments::sweep::sweep(&app.state).await.unwrap(), 1);
    assert!(!api::modules::attachments::files::path(&app.state.data_dir, &orphan).exists());
    assert!(api::modules::attachments::files::path(&app.state.data_dir, &young).exists());
    let trashed = app.call("DELETE", &format!("/api/documents/{doc}"), Some(&roles.member), None).await;
    assert_eq!(trashed.status, 204);
    assert_eq!(
        upload(&app, &roles.member, &doc, "late.png", PNG).await.json()["error"]["code"],
        json!("failed_precondition")
    );
}
