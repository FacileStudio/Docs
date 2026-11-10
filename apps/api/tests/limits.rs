mod common;

use std::net::SocketAddr;
use std::time::Duration;

use common::fixture;
use common::typing::{callouts, nest_one_more_callout, read};
use common::ysync::Peer;
use serde_json::{Value, json};

fn nested_callouts(depth: usize) -> String {
    (0..depth).rev().fold("deep".to_owned(), |inner, level| {
        let fence = "`".repeat(depth + 3 - level);
        format!("{fence}callout tone=\"info\"\n{inner}\n{fence}")
    })
}

fn error(answer: &common::http::Answer) -> (String, String) {
    let body = answer.json();
    (body["error"]["code"].as_str().unwrap().to_owned(), body["error"]["message"].as_str().unwrap().to_owned())
}

#[tokio::test]
async fn markdown_past_the_nesting_limits_is_refused_with_its_line_and_column() {
    let Some(pool) = common::pool("docs_limits_nest").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    fixture::page(&app, &roles.member, None, &format!("intro\n\n{}", nested_callouts(8))).await;
    let body = json!({"title": "x", "markdown": format!("intro\n\n{}", nested_callouts(9))});
    let refused = app.call("POST", "/api/documents", Some(&roles.member), Some(body)).await;
    assert_eq!(refused.status, 400);
    let (code, message) = error(&refused);
    assert_eq!(code, "invalid_argument");
    assert!(message.starts_with("markdown at line ") && message.contains(", column "), "{message}");
    let quotes = json!({"title": "x", "markdown": format!("{}x", "> ".repeat(70))});
    let refused = app.call("POST", "/api/documents", Some(&roles.member), Some(quotes)).await;
    assert!(error(&refused).1.starts_with("markdown at line 1, column "), "{}", refused.text());
}

#[tokio::test]
async fn a_patch_that_matches_too_often_reports_the_count_and_lists_twenty() {
    let Some(pool) = common::pool("docs_limits_patch").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let doc = fixture::page(&app, &roles.member, None, &vec!["item"; 25].join("\n\n")).await;
    let path = format!("/api/documents/{}/ops", fixture::id(&doc));
    let ops = json!({"ops": [{"op": "patch", "find": "item", "replace": "thing"}]});
    let refused = app.call("POST", &path, Some(&roles.member), Some(ops)).await;
    assert_eq!(refused.status, 400);
    let (_, message) = error(&refused);
    assert!(message.starts_with("patch expected 1 match, found 25: "), "{message}");
    assert!(message.contains("and 5 more") && message.matches("block ").count() == 20, "{message}");
}

async fn status(app: &common::App, who: &common::Person, path: &str, body: Value) -> (u16, String) {
    let answer = app.call("POST", path, Some(who), Some(body)).await;
    let code = answer.json()["error"]["code"].as_str().unwrap_or_default().to_owned();
    (answer.status.as_u16(), code)
}

#[tokio::test]
async fn markdown_over_the_cap_is_413_on_create_openui_and_ops() {
    let Some(pool) = common::pool("docs_limits_cap").await else { return };
    let app = common::app_with(pool, &[("MARKDOWN_MAX_BYTES", "1024")]).await;
    let roles = fixture::roles(&app).await;
    let who = &roles.member;
    let over = (413, "resource_exhausted".to_owned());
    let create = |n: usize| json!({"title": "x", "markdown": "a".repeat(n)});
    assert_eq!(status(&app, who, "/api/documents", create(1025)).await, over);
    assert_eq!(status(&app, who, "/api/documents", create(1024)).await.0, 201);
    let openui = json!({"title": "x", "source": format!("root = Text(\"{}\")", "a".repeat(1024))});
    assert_eq!(status(&app, who, "/api/documents/import-openui", openui).await, over);
    let doc = fixture::page(&app, who, None, "one").await;
    let path = format!("/api/documents/{}/ops", fixture::id(&doc));
    let half = "b".repeat(600);
    let two = json!({"ops": [{"op": "append", "markdown": half}, {"op": "prepend", "markdown": half}]});
    assert_eq!(status(&app, who, &path, two).await, over);
    let patch = json!({"ops": [{"op": "patch", "find": "one", "replace": "c".repeat(400), "expect": 3}]});
    assert_eq!(status(&app, who, &path, patch).await, over);
    let read = app.call("GET", &format!("/api/documents/{}", fixture::id(&doc)), Some(who), None).await.json();
    assert_eq!(read["version"], json!(1));
}

#[tokio::test]
async fn an_update_past_the_schema_2_nesting_limits_is_reverted_and_closed_with_1008() {
    let Some(pool) = common::pool("docs_rooms_nesting").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::page(&app, &roles.member, None, "safe").await);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/api/documents/{id}/ws", listener.local_addr().unwrap());
    let router = app.router.clone().into_make_service_with_connect_info::<SocketAddr>();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let mut peer = Peer::new();
    peer.connect(&url, &roles.member.token).await;
    let eight = callouts(8);
    peer.edit(|txn, fragment| encre::yjs::write(txn, fragment, &eight).unwrap()).await;
    peer.pump(Duration::from_millis(500)).await;
    assert_eq!(peer.closed, None);
    peer.edit(nest_one_more_callout).await;
    peer.pump(Duration::from_millis(800)).await;
    assert_eq!(peer.closed, Some(1008));
    let mut fresh = Peer::new();
    fresh.connect(&url, &roles.member.token).await;
    assert_eq!(read(&fresh.doc), encre::compact(&eight));
}
