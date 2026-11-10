mod common;

use common::fixture;
use common::upload::{PNG, upload};
use serde_json::json;

#[tokio::test]
async fn a_share_link_shows_its_page_and_only_its_subtree() {
    let Some(pool) = common::pool("docs_share").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let root = fixture::id(&fixture::page(&app, &roles.owner, Some(&roles.space), "root page").await);
    let child = json!({"title": "Child", "parent_id": root, "markdown": "child page"});
    let child = fixture::id(&app.call("POST", "/api/documents", Some(&roles.owner), Some(child)).await.json());
    let other = fixture::id(&fixture::page(&app, &roles.owner, Some(&roles.space), "elsewhere").await);
    let path = format!("/api/documents/{root}/share");
    assert_eq!(app.call("POST", &path, Some(&roles.member), Some(json!({"children": true}))).await.status, 403);
    let link = app.call("POST", &path, Some(&roles.owner), Some(json!({"children": true}))).await;
    assert_eq!(link.status, 201);
    let token = link.json()["url"].as_str().unwrap().trim_start_matches("/s/").to_owned();
    let public = app.call("GET", &format!("/api/public/{token}"), None, None).await.json();
    assert_eq!(public["children"][0]["id"], json!(child));
    assert_eq!(public["children"][0]["title"], json!("Child"));
    assert_eq!(app.call("GET", &format!("/api/public/{token}/documents/{child}"), None, None).await.status, 200);
    assert_eq!(app.call("GET", &format!("/api/public/{token}/documents/{other}"), None, None).await.status, 404);
    let listed = app.call("GET", &path, Some(&roles.owner), None).await.json();
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert!(listed[0].get("token").is_none());
    let id = link.json()["id"].as_str().unwrap().to_owned();
    assert_eq!(app.call("DELETE", &format!("/api/share/{id}"), Some(&roles.outsider), None).await.status, 404);
    assert_eq!(app.call("DELETE", &format!("/api/share/{id}"), Some(&roles.admin), None).await.status, 204);
    assert_eq!(app.call("GET", &format!("/api/public/{token}"), None, None).await.status, 404);
}

#[tokio::test]
async fn a_shared_page_carries_its_images_with_the_token() {
    let Some(pool) = common::pool("docs_share_images").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let doc = fixture::id(&fixture::page(&app, &roles.owner, None, "pic").await);
    let url = upload(&app, &roles.owner, &doc, "a.png", PNG).await.json()["url"].as_str().unwrap().to_owned();
    let ops = json!({"ops": [{"op": "append", "markdown": format!("![dot]({url})")}]});
    app.call("POST", &format!("/api/documents/{doc}/ops"), Some(&roles.owner), Some(ops)).await;
    let link = app.call("POST", &format!("/api/documents/{doc}/share"), Some(&roles.owner), Some(json!({}))).await;
    let token = link.json()["url"].as_str().unwrap().trim_start_matches("/s/").to_owned();
    let public = app.call("GET", &format!("/api/public/{token}"), None, None).await.json();
    let src = public["content"]["content"][1]["attrs"]["src"].as_str().unwrap().to_owned();
    assert_eq!(src, format!("{url}?share={token}"));
    assert_eq!(app.call("GET", &src, None, None).await.status, 200);
    assert_eq!(app.call("GET", &format!("{url}?share=wrong"), None, None).await.status, 404);
}

#[tokio::test]
async fn the_public_routes_are_rate_limited_per_client() {
    let Some(pool) = common::pool("docs_share_limit").await else { return };
    let app = common::app(pool).await;
    let mut statuses = Vec::new();
    for _ in 0..40 {
        statuses.push(app.call("GET", "/api/public/nope", None, None).await.status.as_u16());
    }
    assert!(statuses.contains(&404));
    assert_eq!(*statuses.last().unwrap(), 429, "{statuses:?}");
}
