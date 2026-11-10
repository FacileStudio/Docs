mod common;

use std::sync::Arc;

use common::mcp::{Served, args, served};
use serde_json::{Value, json};
use tronc::mcptest::{self, ClientOptions, ConfirmCase, Target, call, connect, decode};

async fn version(app: &common::App, id: &str) -> i64 {
    sqlx::query_scalar("SELECT version FROM documents WHERE id = $1::uuid").bind(id).fetch_one(&app.pool).await.unwrap()
}

async fn runs(state: api::state::AppState, id: String) -> usize {
    let v: i64 = sqlx::query_scalar("SELECT version FROM documents WHERE id = $1::uuid")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .unwrap();
    (v - 1) as usize
}

async fn fresh(state: api::state::AppState, router: axum::Router, token: String) -> Target {
    let body = json!({"title": "Target", "markdown": "before"});
    let doc = common::send(&router, "POST", "/api/documents", Some(&token), Some(body)).await.json();
    let id = doc["id"].as_str().unwrap().to_owned();
    let target = args(json!({"document_id": id, "markdown": "after", "expect_version": 1}));
    let count = move || -> mcptest::Fut<usize> { Box::pin(runs(state.clone(), id.clone())) };
    Target { args: target, runs: Arc::new(count) }
}

fn replace_case(served: &Served) -> ConfirmCase {
    let (state, router, token) = (served.app.state.clone(), served.app.router.clone(), served.person.token.clone());
    let setup = move || -> mcptest::Fut<Target> { Box::pin(fresh(state.clone(), router.clone(), token.clone())) };
    ConfirmCase { tool: "docs_replace".into(), setup: Arc::new(setup) }
}

#[tokio::test]
async fn the_mcp_endpoint_passes_tronc_conformance_with_docs_replace_confirmed() {
    let Some(served) = served("docs_mcp_kit").await else { return };
    let user = served.person.user_id;
    let token = Arc::new(move |scopes: &[&str]| {
        let scopes = if scopes.is_empty() { "docs.read,docs.write".to_owned() } else { scopes.join(",") };
        format!("{user}|{scopes}")
    });
    let confirm = vec![replace_case(&served)];
    mcptest::run(&mcptest::Config { url: served.url.clone(), token, confirm }).await;
}

#[tokio::test]
async fn agents_read_markdown_edit_blocks_and_confirm_a_replace_on_the_page() {
    let Some(served) = served("docs_mcp_tools").await else { return };
    let token = format!("{}|docs.read,docs.write", served.person.user_id);
    let client = connect(&served.url, &token, ClientOptions::default()).await;
    let created: Value =
        decode(&call(&client, "docs_create", args(json!({"title": "Plan", "markdown": "# Plan"}))).await);
    let id = created["id"].as_str().unwrap().to_owned();
    let blocks: Value = decode(&call(&client, "docs_read_blocks", args(json!({"document_id": id}))).await);
    let heading = blocks["blocks"][0]["id"].as_str().unwrap().to_owned();
    let ops = json!({"document_id": id, "ops": [{"op": "insert_after", "block_id": heading, "markdown": "Step one."}]});
    let edited: Value = decode(&call(&client, "docs_edit", args(ops)).await);
    assert_eq!(edited["markdown"], json!("# Plan\n\nStep one."));
    let read: Value = decode(&call(&client, "docs_read", args(json!({"document_id": id}))).await);
    assert_eq!((read["markdown"].clone(), read["version"].clone()), (json!("# Plan\n\nStep one."), json!(2)));
    let found: Value = decode(&call(&client, "docs_search", args(json!({"query": "step"}))).await);
    assert_eq!(found["results"][0]["id"], json!(id));
    confirm_on_the_page(&served, &client, &id).await;
}

async fn confirm_on_the_page(served: &Served, client: &tronc::mcptest::Client, id: &str) {
    let ask = json!({"document_id": id, "markdown": "Replaced.", "expect_version": 2});
    let pending: Value = decode(&call(client, "docs_replace", args(ask)).await);
    assert_eq!(pending["status"], json!("pending_confirmation"));
    let action = pending["confirm_url"].as_str().unwrap().rsplit('=').next().unwrap().to_owned();
    assert_eq!(version(&served.app, id).await, 2);
    let list = served.app.call("GET", "/api/mcp/actions", Some(&served.person), None).await.json();
    assert_eq!(list[0]["id"], json!(action));
    let path = format!("/api/mcp/actions/{action}/confirm");
    let done = served.app.call("POST", &path, Some(&served.person), None).await;
    assert_eq!(done.json(), json!({"status": "done"}), "{}", done.text());
    let read: Value = decode(&call(client, "docs_read", args(json!({"document_id": id}))).await);
    assert_eq!(read["markdown"], json!("Replaced."));
}
