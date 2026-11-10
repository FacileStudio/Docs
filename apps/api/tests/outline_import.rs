mod common;

use std::collections::HashMap;
use std::sync::Arc;

use api::outline::{self, Args};
use axum::Router;
use axum::extract::{Json, State};
use axum::routing::{get, post};
use serde_json::{Value, json};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/testdata/outline");

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("{FIXTURES}/{name}")).unwrap()
}

async fn exported(State(texts): State<Arc<HashMap<String, String>>>, Json(body): Json<Value>) -> Json<Value> {
    Json(json!({"data": texts[body["id"].as_str().unwrap()], "status": 200, "ok": true}))
}

async fn outline(author: &str) -> String {
    let listed = fixture("documents.list.json").replace("yann@facile.test", author);
    let texts: HashMap<String, String> = (1..=3)
        .map(|i| (format!("d0000000-0000-4000-8000-00000000000{i}"), fixture(&format!("export.d{i}.md"))))
        .collect();
    let list = |name: &'static str| post(move || async move { fixture(name) });
    let app = Router::new()
        .route("/api/collections.list", list("collections.list.json"))
        .route("/api/collections.documents", list("collections.documents.json"))
        .route("/api/documents.list", post(move || std::future::ready(listed.clone())))
        .route("/api/documents.export", post(exported))
        .route("/api/attachments.redirect", get(|| async { common::upload::PNG.to_vec() }))
        .with_state(Arc::new(texts));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    url
}

#[tokio::test]
async fn the_dry_run_reports_the_tree_and_what_it_cannot_convert_and_writes_nothing() {
    let Some(pool) = common::pool("docs_outline_dry").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = common::fixture::roles(&app).await;
    let args = Args {
        url: outline("nobody@example.test").await,
        token: "t".into(),
        space: roles.space.clone(),
        dry_run: true,
    };
    let report = outline::run(&pool, &app.state.data_dir, &args).await.unwrap();
    assert!(
        report.starts_with("4 pages\n- Engineering (0 attachments)\n  - Deploy (1 attachments)\n    - Rollback"),
        "{report}"
    );
    assert!(report.contains("1 unconverted constructs\n  Glossary: notice `:::unknownstyle`"), "{report}");
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM documents WHERE space_id = $1")
        .bind(&roles.space)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn an_import_keeps_parents_order_dates_authors_links_callouts_and_attachments() {
    let Some(pool) = common::pool("docs_outline").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = common::fixture::roles(&app).await;
    let email: String = sqlx::query_scalar("SELECT email FROM porte_users WHERE id = $1")
        .bind(roles.owner.user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let args = Args { url: outline(&email).await, token: "t".into(), space: roles.space.clone(), dry_run: false };
    outline::run(&pool, &app.state.data_dir, &args).await.unwrap();
    let tree =
        app.call("GET", &format!("/api/documents?space_id={}", roles.space), Some(&roles.member), None).await.json();
    let mut titles: Vec<&str> = tree.as_array().unwrap().iter().map(|e| e["title"].as_str().unwrap()).collect();
    titles.sort();
    assert_eq!(titles, ["Deploy", "Engineering", "Glossary", "Rollback"]);
    let by = |t: &str| tree.as_array().unwrap().iter().find(|e| e["title"] == t).unwrap().clone();
    assert_eq!(by("Rollback")["parent_id"], by("Deploy")["id"]);
    assert_eq!(by("Glossary")["parent_id"], by("Engineering")["id"]);
    assert_eq!((by("Deploy")["position"].clone(), by("Glossary")["position"].clone()), (json!(1.0), json!(2.0)));
    let deploy = by("Deploy")["id"].as_str().unwrap().to_owned();
    check_deploy(&app, &roles, &deploy, by("Rollback")["id"].as_str().unwrap()).await;
}

async fn check_deploy(app: &common::App, roles: &common::fixture::Roles, deploy: &str, rollback: &str) {
    let path = format!("/api/documents/{deploy}?format=markdown");
    let md = app.call("GET", &path, Some(&roles.member), None).await.json()["markdown"].as_str().unwrap().to_owned();
    assert!(
        md.contains("````callout tone=\"warning\"\nRun the gate **first**.\n````")
            || md.contains("```callout tone=\"warning\"\nRun the gate **first**.\n```"),
        "{md}"
    );
    assert!(md.contains(&format!("[rollback](/d/{rollback})")), "{md}");
    assert!(md.contains("breaks.\\\nA second line"), "{md}");
    let src = md.split("![diagram](").nth(1).unwrap().split(')').next().unwrap().to_owned();
    assert_eq!(app.call("GET", &src, Some(&roles.member), None).await.bytes, common::upload::PNG);
    let (created, author): (time::OffsetDateTime, Option<i64>) =
        sqlx::query_as("SELECT created_at, created_by FROM documents WHERE id = $1::uuid")
            .bind(deploy)
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(created.unix_timestamp(), 1740909600);
    assert_eq!(author, Some(roles.owner.user_id));
}
