mod common;

use common::fixture;
use serde_json::json;

#[tokio::test]
async fn a_page_reads_back_as_json_and_as_markdown() {
    let Some(pool) = common::pool("docs_pages").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let doc = fixture::page(&app, &roles.member, Some(&roles.space), "# Plan\n\nShip **it**.").await;
    assert_eq!(doc["content"]["content"][0]["type"], json!("heading"));
    assert!(doc["content"]["content"][0]["attrs"]["id"].is_string());
    assert_eq!(doc["access"], json!("editor"));
    let id = fixture::id(&doc);
    let md = app.call("GET", &format!("/api/documents/{id}?format=markdown"), Some(&roles.member), None).await.json();
    assert_eq!(md["markdown"], json!("# Plan\n\nShip **it**."));
    assert!(md.get("content").is_none());
    let admin = app.call("GET", &format!("/api/documents/{id}"), Some(&roles.admin), None).await.json();
    assert_eq!(admin["access"], json!("manager"));
    let bad = json!({"title": "x", "content": {"type": "doc", "content": []}});
    assert_eq!(app.call("POST", "/api/documents", Some(&roles.member), Some(bad)).await.status, 400);
    let unknown = json!({"title": "x", "colour": "red"});
    assert_eq!(app.call("POST", "/api/documents", Some(&roles.member), Some(unknown)).await.status, 400);
}

#[tokio::test]
async fn ops_apply_in_order_bump_the_version_and_refuse_atomically() {
    let Some(pool) = common::pool("docs_ops").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let doc = fixture::page(&app, &roles.member, None, "one").await;
    let path = format!("/api/documents/{}/ops", fixture::id(&doc));
    let ops =
        json!({"ops": [{"op": "append", "markdown": "three"}, {"op": "patch", "find": "three", "replace": "two"}]});
    let answer = app.call("POST", &path, Some(&roles.member), Some(ops)).await.json();
    assert_eq!(answer, json!({"version": 2, "markdown": "one\n\ntwo", "saved": true}));
    let refused = json!({"ops": [{"op": "append", "markdown": "x"}, {"op": "delete_block", "block_id": "0000000000"}]});
    let answer = app.call("POST", &path, Some(&roles.member), Some(refused)).await;
    assert_eq!(answer.status, 400);
    assert!(answer.json()["error"]["message"].as_str().unwrap().contains("read_blocks"));
    let stale = json!({"ops": [{"op": "replace_all", "markdown": "x", "expect_version": 1}]});
    assert_eq!(app.call("POST", &path, Some(&roles.member), Some(stale)).await.status, 412);
}

#[tokio::test]
async fn trash_takes_the_subtree_and_restore_brings_it_back() {
    let Some(pool) = common::pool("docs_trash").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let parent = fixture::id(&fixture::page(&app, &roles.owner, Some(&roles.space), "parent").await);
    let child = json!({"title": "Child", "parent_id": parent});
    let child = fixture::id(&app.call("POST", "/api/documents", Some(&roles.owner), Some(child)).await.json());
    let tree = format!("/api/documents?space_id={}", roles.space);
    assert_eq!(app.call("GET", &tree, Some(&roles.owner), None).await.json().as_array().unwrap().len(), 2);
    assert_eq!(app.call("DELETE", &format!("/api/documents/{parent}"), Some(&roles.owner), None).await.status, 204);
    assert_eq!(app.call("GET", &tree, Some(&roles.owner), None).await.json(), json!([]));
    let trashed = app.call("GET", &format!("{tree}&trashed=true"), Some(&roles.owner), None).await.json();
    assert_eq!(trashed.as_array().unwrap().len(), 2);
    let restored = app.call("POST", &format!("/api/documents/{child}/restore"), Some(&roles.owner), None).await.json();
    assert_eq!(restored["parent_id"], json!(null));
    let restore = app.call("POST", &format!("/api/documents/{parent}/restore"), Some(&roles.owner), None).await;
    assert_eq!(restore.status, 200);
    assert_eq!(
        app.call("DELETE", &format!("/api/documents/{parent}/permanent"), Some(&roles.owner), None).await.status,
        412
    );
}

#[tokio::test]
async fn move_places_a_page_between_its_new_neighbours_and_refuses_cycles() {
    let Some(pool) = common::pool("docs_move").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let space = Some(roles.space.as_str());
    let a = fixture::id(&fixture::page(&app, &roles.owner, space, "a").await);
    let b = fixture::id(&fixture::page(&app, &roles.owner, space, "b").await);
    let c = fixture::id(&fixture::page(&app, &roles.owner, space, "c").await);
    let moved =
        app.call("POST", &format!("/api/documents/{c}/move"), Some(&roles.owner), Some(json!({"before": b}))).await;
    assert_eq!(moved.json()["position"], json!(1.5));
    let under = json!({"parent_id": a});
    let moved = app.call("POST", &format!("/api/documents/{b}/move"), Some(&roles.owner), Some(under)).await.json();
    assert_eq!(moved["parent_id"], json!(a));
    let cycle = json!({"parent_id": b});
    assert_eq!(
        app.call("POST", &format!("/api/documents/{a}/move"), Some(&roles.owner), Some(cycle)).await.status,
        400
    );
    let personal = fixture::id(&fixture::page(&app, &roles.owner, None, "mine").await);
    let across = json!({"parent_id": personal});
    assert_eq!(
        app.call("POST", &format!("/api/documents/{a}/move"), Some(&roles.owner), Some(across)).await.status,
        400
    );
}

#[tokio::test]
async fn a_revision_restores_the_content_it_saved() {
    let Some(pool) = common::pool("docs_revisions").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::page(&app, &roles.member, None, "first").await);
    let ops = json!({"ops": [{"op": "replace_all", "markdown": "second", "expect_version": 1}]});
    let applied = app.call("POST", &format!("/api/documents/{id}/ops"), Some(&roles.member), Some(ops)).await.json();
    assert_eq!(applied["saved"], json!(true));
    let list = app.call("GET", &format!("/api/documents/{id}/revisions"), Some(&roles.member), None).await.json();
    let rev = list[0]["id"].as_str().unwrap();
    assert!(list[0].get("content").is_none());
    let one = app.call("GET", &format!("/api/documents/{id}/revisions/{rev}"), Some(&roles.member), None).await.json();
    assert_eq!(one["content"]["content"][0]["content"][0]["text"], json!("second"));
    let ops = json!({"ops": [{"op": "replace_all", "markdown": "third", "expect_version": 2}]});
    app.call("POST", &format!("/api/documents/{id}/ops"), Some(&roles.member), Some(ops)).await;
    let path = format!("/api/documents/{id}/revisions/{rev}/restore");
    let restored = app.call("POST", &path, Some(&roles.member), None).await.json();
    assert_eq!(restored["content"]["content"][0]["content"][0]["text"], json!("second"));
    assert_eq!(restored["version"], json!(4));
}

#[tokio::test]
async fn search_ranks_and_highlights_inside_the_scope() {
    let Some(pool) = common::pool("docs_search").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let space = Some(roles.space.as_str());
    fixture::page(&app, &roles.member, space, "The deployment checklist for la ruche").await;
    fixture::page(&app, &roles.member, space, "Unrelated notes").await;
    let path = format!("/api/search?q=deployment&space_id={}", roles.space);
    let hits = app.call("GET", &path, Some(&roles.member), None).await.json();
    assert_eq!(hits.as_array().unwrap().len(), 1);
    assert!(hits[0]["snippet"].as_str().unwrap().contains("<b>deployment</b>"), "{hits}");
    assert_eq!(app.call("GET", "/api/search?q=", Some(&roles.member), None).await.status, 400);
}

#[tokio::test]
async fn the_ops_subcommand_edits_a_page_through_its_room() {
    let Some(pool) = common::pool("docs_operator_ops").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let doc = fixture::id(&fixture::page(&app, &roles.owner, Some(&roles.space), "old link").await);
    let file = common::data_dir().join(format!("ops-{doc}.json"));
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, r#"[{"op": "patch", "find": "old", "replace": "new", "expect": 1}]"#).unwrap();
    let config =
        api::env::Config::from_vars(&tronc::env::Vars::from_pairs(vec![("DATABASE_URL", "postgres://x/y")])).unwrap();
    let argv: Vec<String> = ["api", "ops", &doc, file.to_str().unwrap()].map(String::from).to_vec();
    api::operator::command(&argv, &config, &pool).await.unwrap().unwrap();
    let page = app.call("GET", &format!("/api/documents/{doc}?format=markdown"), Some(&roles.owner), None).await;
    assert!(page.text().contains("new link"), "{}", page.text());
    let usage = ["api", "ops", &doc].map(String::from).to_vec();
    assert!(api::operator::command(&usage, &config, &pool).await.unwrap().is_err());
    assert!(api::operator::command(&["api".to_owned()], &config, &pool).await.is_none());
}
