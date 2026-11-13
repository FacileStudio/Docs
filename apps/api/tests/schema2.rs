mod common;

use api::modules::documents::{content, markdown};
use common::fixture;
use serde_json::{Value, json};

const V1: &str = "# Plan\n\n````callout tone=\"warning\" title=\"Before\"\nRun the gate.\n````\n\n- [x] done\n\n\
```stat\n{\"label\": \"Revenue\", \"value\": \"12\", \"delta\": \"+4%\"}\n```";

const V2: &str = "Ship ==now== [badge:success Shipped] [status:done]\n\n```mermaid\nflowchart LR\n  A --> B\n```\n\n\
````columns\n```column title=\"Pros\" tone=\"success\"\nFast\n```\n\n```column title=\"Cons\" tone=\"danger\"\nLong\n```\n\
````\n\n> [!TIP]\n> Read it";

async fn insert_v1(app: &common::App, owner: i64, stored: &Value) -> String {
    let id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO documents (owner_id, title, schema_version, content, created_by, updated_by) \
         VALUES ($1, 'Old', 1, $2, $1, $1) RETURNING id",
    )
    .bind(owner)
    .bind(sqlx::types::Json(stored))
    .fetch_one(&app.pool)
    .await
    .unwrap();
    id.to_string()
}

async fn schema_of(app: &common::App, id: &str) -> i32 {
    sqlx::query_scalar("SELECT schema_version FROM documents WHERE id = $1::uuid")
        .bind(id)
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_schema_1_row_is_served_unchanged_until_a_write_stores_it_as_schema_2() {
    let Some(pool) = common::pool("docs_schema_v1").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let prepared = content::prepare(markdown::from_markdown(V1).unwrap()).unwrap();
    let id = insert_v1(&app, roles.member.user_id, &prepared.json).await;
    let read = app.call("GET", &format!("/api/documents/{id}"), Some(&roles.member), None).await.json();
    assert_eq!((read["schema_version"].clone(), read["content"].clone()), (json!(1), prepared.json.clone()));
    let md = app.call("GET", &format!("/api/documents/{id}?format=markdown"), Some(&roles.member), None).await;
    assert_eq!(md.json()["markdown"], json!(encre::markdown::serialize(&prepared.node)));
    assert_eq!(schema_of(&app, &id).await, 1);
    let ops = json!({"ops": [{"op": "append", "markdown": "Added ==later==."}]});
    let path = format!("/api/documents/{id}/ops");
    assert_eq!(app.call("POST", &path, Some(&roles.member), Some(ops)).await.status, 200);
    assert_eq!(schema_of(&app, &id).await, 2);
    let after = app.call("GET", &format!("/api/documents/{id}"), Some(&roles.member), None).await.json();
    let blocks = after["content"]["content"].as_array().unwrap();
    assert_eq!(blocks[..blocks.len() - 1], prepared.json["content"].as_array().unwrap()[..]);
}

#[tokio::test]
async fn a_new_page_stores_schema_2_and_keeps_every_schema_2_construct() {
    let Some(pool) = common::pool("docs_schema_v2").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let doc = fixture::page(&app, &roles.member, Some(&roles.space), V2).await;
    assert_eq!(doc["schema_version"], json!(encre::SCHEMA_VERSION));
    let types = doc["content"]["content"].as_array().unwrap().iter().map(|b| b["type"].clone()).collect::<Vec<_>>();
    assert_eq!(types, [json!("paragraph"), json!("diagram"), json!("columns"), json!("callout")]);
    let inline = doc["content"]["content"][0]["content"].to_string();
    assert!(inline.contains("\"highlight\"") && inline.contains("\"badge\""), "{inline}");
    let id = fixture::id(&doc);
    let md = app.call("GET", &format!("/api/documents/{id}?format=markdown"), Some(&roles.member), None).await;
    let md = md.json()["markdown"].as_str().unwrap().to_owned();
    for wanted in ["==now==", "[badge:success Shipped]", "[status:done]", "```mermaid", "````columns"] {
        assert!(md.contains(wanted), "{wanted} missing from {md}");
    }
}

#[tokio::test]
async fn a_room_opening_a_schema_1_ydoc_migrates_it_and_stores_schema_2() {
    let Some(pool) = common::pool("docs_schema_ydoc").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let prepared = content::prepare(markdown::from_markdown(V1).unwrap()).unwrap();
    let id = insert_v1(&app, roles.member.user_id, &prepared.json).await;
    sqlx::query("UPDATE documents SET ydoc = $2 WHERE id = $1::uuid")
        .bind(&id)
        .bind(encre::yjs::to_update(&prepared.node).unwrap())
        .execute(&app.pool)
        .await
        .unwrap();
    let url = format!("{}/api/documents/{id}/ws", app.serve().await);
    let mut peer = common::ysync::Peer::new();
    peer.connect(&url, &roles.member.token).await;
    assert_eq!(common::typing::read(&peer.doc), prepared.node);
    assert_eq!(schema_of(&app, &id).await, encre::SCHEMA_VERSION as i32);
    peer.disconnect();
}
