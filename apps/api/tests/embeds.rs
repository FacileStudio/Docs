mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use api::modules::assistant::{self, Assistant};
use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use serde_json::{Value, json};

fn query(url: &str) -> String {
    let encoded: String = url
        .bytes()
        .map(|b| match b.is_ascii_alphanumeric() {
            true => char::from(b).to_string(),
            false => format!("%{b:02X}"),
        })
        .collect();
    format!("/api/embeds/resolve?url={encoded}")
}

#[tokio::test]
async fn unfurls_refuse_plain_http_private_literals_metadata_and_credentials() {
    let Some(pool) = common::pool("docs_embeds_refuse").await else { return };
    let app = common::app(pool).await;
    let who = app.person("embedder").await;
    assert_eq!(app.call("GET", &query("https://example.com"), None, None).await.status, 401);
    let refused = [
        "http://example.com/",
        "https://127.0.0.1/",
        "https://10.0.0.8/",
        "https://192.168.1.1/",
        "https://169.254.169.254/latest/meta-data/",
        "https://[::1]/",
        "https://[fd00:ec2::254]/",
        "https://0x7f.1/",
        "https://user:pass@example.com/",
        "file:///etc/passwd",
        "not a link",
    ];
    for url in refused {
        let answer = app.call("GET", &query(url), Some(&who), None).await;
        assert_eq!(answer.status, 400, "{url}: {}", answer.text());
        assert_eq!(answer.json()["error"]["code"], json!("invalid_argument"), "{url}");
    }
    let long = format!("https://example.com/{}", "a".repeat(2048));
    assert_eq!(app.call("GET", &query(&long), Some(&who), None).await.status, 400);
}

#[tokio::test]
async fn a_name_that_resolves_to_loopback_is_never_contacted() {
    let Some(pool) = common::pool("docs_embeds_dns").await else { return };
    let app = common::app(pool).await;
    let who = app.person("embedder").await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let reached = Arc::new(AtomicUsize::new(0));
    let count = reached.clone();
    tokio::spawn(async move {
        while listener.accept().await.is_ok() {
            count.fetch_add(1, Ordering::SeqCst);
        }
    });
    let answer = app.call("GET", &query(&format!("https://localhost:{port}/")), Some(&who), None).await;
    assert_eq!(answer.status, 200, "{}", answer.text());
    let body = answer.json();
    assert_eq!(body["title"], json!("localhost"));
    assert_eq!((body["image"].clone(), body["description"].clone()), (Value::Null, Value::Null));
    assert_eq!(reached.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn the_assistant_route_answers_bulle_without_registre() {
    let Some(pool) = common::pool("docs_assistant").await else { return };
    let app = common::app(pool).await;
    let who = app.person("asker").await;
    assert_eq!(app.call("GET", "/api/assistant", None, None).await.status, 401);
    let answer = app.call("GET", "/api/assistant", Some(&who), None).await;
    assert_eq!(answer.status, 200);
    assert_eq!(answer.json(), json!({"name": "Bulle", "accent": null}));
}

async fn bulle(status: StatusCode, profile: Value) -> String {
    let route = get(move |headers: HeaderMap| async move {
        let bearer = headers.get("authorization").and_then(|v| v.to_str().ok()) == Some("Bearer tok");
        match bearer {
            true => (status, Json(profile)),
            false => (StatusCode::UNAUTHORIZED, Json(json!({}))),
        }
    });
    let router = axum::Router::new().route("/api/assistant", route);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    url
}

async fn asked(url: &str) -> Assistant {
    assistant::ask(&assistant::client(url).unwrap(), "tok").await
}

#[tokio::test]
async fn the_assistant_profile_is_read_from_bulle_and_any_failure_falls_back() {
    let named = bulle(StatusCode::OK, json!({"name": " Nova ", "accent": "#7c3aed", "archetype": "x"})).await;
    assert_eq!(asked(&named).await, Assistant { name: "Nova".into(), accent: Some("#7c3aed".into()) });
    let blank = bulle(StatusCode::OK, json!({"name": "  "})).await;
    assert_eq!(asked(&blank).await, Assistant::default());
    for status in [StatusCode::PAYMENT_REQUIRED, StatusCode::NOT_FOUND, StatusCode::INTERNAL_SERVER_ERROR] {
        let failing = bulle(status, json!({"name": "Nova"})).await;
        assert_eq!(asked(&failing).await, Assistant::default(), "{status}");
    }
    let wrong = assistant::ask(&assistant::client(&named).unwrap(), "other").await;
    assert_eq!(wrong, Assistant::default());
    assert_eq!(asked("http://127.0.0.1:1").await, Assistant::default());
}
