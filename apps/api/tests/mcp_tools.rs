mod common;

use common::mcp::{args, refused, served};
use serde_json::{Value, json};
use tronc::mcptest::{call, connect, decode};

async fn page(client: &tronc::mcptest::Client, title: &str, parent: Option<&str>) -> String {
    let input = json!({"title": title, "markdown": "Body.", "parent_id": parent});
    let created: Value = decode(&call(client, "docs_create", args(input)).await);
    created["id"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn docs_update_renames_a_page_twice_to_the_same_state_and_names_an_empty_call() {
    let Some(served) = served("docs_mcp_update").await else { return };
    let client =
        connect(&served.url, &format!("{}|docs.read,docs.write", served.person.user_id), Default::default()).await;
    let id = page(&client, "Draft", None).await;
    let input = json!({"document_id": id, "title": "  Roadmap ", "icon": "🗺️"});
    let first: Value = decode(&call(&client, "docs_update", args(input.clone())).await);
    let second: Value = decode(&call(&client, "docs_update", args(input)).await);
    assert_eq!((first["title"].clone(), first["icon"].clone()), (json!("Roadmap"), json!("🗺️")));
    assert_eq!((second["title"].clone(), second["version"].clone()), (json!("Roadmap"), first["version"].clone()));
    assert!(refused(&client, "docs_update", json!({"document_id": id})).await.contains("title, icon"));
    let long = json!({"document_id": id, "title": "x".repeat(201)});
    let message = refused(&client, "docs_update", long).await;
    assert!(message.contains("longer than 200"), "{message}");
}

#[tokio::test]
async fn docs_move_nests_a_page_orders_it_and_refuses_a_cycle() {
    let Some(served) = served("docs_mcp_move").await else { return };
    let client =
        connect(&served.url, &format!("{}|docs.read,docs.write", served.person.user_id), Default::default()).await;
    let (parent, first, child) =
        (page(&client, "Parent", None).await, page(&client, "First", None).await, page(&client, "Child", None).await);
    let moved: Value =
        decode(&call(&client, "docs_move", args(json!({"document_id": child, "parent_id": parent}))).await);
    assert_eq!(moved["parent_id"], json!(parent));
    let into = json!({"document_id": first, "parent_id": parent, "before_id": child});
    decode::<Value>(&call(&client, "docs_move", args(into)).await);
    let tree: Value = decode(&call(&client, "docs_list_tree", args(json!({}))).await);
    let titles: Vec<&str> = tree["pages"].as_array().unwrap().iter().map(|p| p["title"].as_str().unwrap()).collect();
    assert_eq!(titles, ["Parent", "First", "Child"]);
    let cycle = json!({"document_id": parent, "parent_id": child});
    assert!(refused(&client, "docs_move", cycle).await.contains("inside itself"));
    let bad = json!({"document_id": child, "parent_id": "nope"});
    assert!(refused(&client, "docs_move", bad).await.contains("parent_id"));
}

#[tokio::test]
async fn docs_trash_takes_the_subtree_and_is_idempotent() {
    let Some(served) = served("docs_mcp_trash").await else { return };
    let client =
        connect(&served.url, &format!("{}|docs.read,docs.write", served.person.user_id), Default::default()).await;
    let parent = page(&client, "Old", None).await;
    page(&client, "Older", Some(&parent)).await;
    for _ in 0..2 {
        let trashed: Value = decode(&call(&client, "docs_trash", args(json!({"document_id": parent}))).await);
        assert_eq!(trashed["trashed"], json!(true));
    }
    let tree: Value = decode(&call(&client, "docs_list_tree", args(json!({}))).await);
    assert_eq!(tree["pages"], json!([]));
    let outsider = served.app.person("outsider").await;
    let other = connect(&served.url, &format!("{}|docs.read,docs.write", outsider.user_id), Default::default()).await;
    let message = refused(&other, "docs_trash", json!({"document_id": parent})).await;
    assert!(message.contains("not found") && message.contains("docs_list_trash"), "{message}");
}

#[tokio::test]
async fn docs_restore_brings_a_page_back_and_answers_a_live_page_unchanged() {
    let Some(served) = served("docs_mcp_restore").await else { return };
    let client =
        connect(&served.url, &format!("{}|docs.read,docs.write", served.person.user_id), Default::default()).await;
    let id = page(&client, "Back", None).await;
    decode::<Value>(&call(&client, "docs_trash", args(json!({"document_id": id}))).await);
    for _ in 0..2 {
        let restored: Value = decode(&call(&client, "docs_restore", args(json!({"document_id": id}))).await);
        assert_eq!((restored["id"].clone(), restored["trashed"].clone()), (json!(id), json!(false)));
    }
    let read: Value = decode(&call(&client, "docs_read", args(json!({"document_id": id}))).await);
    assert_eq!(read["markdown"], json!("Body."));
}

#[tokio::test]
async fn docs_list_trash_pages_through_the_trash_with_a_cursor() {
    let Some(served) = served("docs_mcp_list_trash").await else { return };
    let client =
        connect(&served.url, &format!("{}|docs.read,docs.write", served.person.user_id), Default::default()).await;
    for title in ["A", "B", "C"] {
        let id = page(&client, title, None).await;
        decode::<Value>(&call(&client, "docs_trash", args(json!({"document_id": id}))).await);
    }
    page(&client, "Live", None).await;
    let first: Value = decode(&call(&client, "docs_list_trash", args(json!({"limit": 2}))).await);
    assert_eq!((first["pages"].as_array().unwrap().len(), first["total"].clone()), (2, json!(3)));
    assert!(first["pages"][0]["trashed_at"].as_str().is_some_and(|at| !at.is_empty()));
    let cursor = first["next_cursor"].as_str().unwrap();
    let rest: Value = decode(&call(&client, "docs_list_trash", args(json!({"limit": 2, "cursor": cursor}))).await);
    assert_eq!((rest["pages"].as_array().unwrap().len(), rest.get("next_cursor")), (1, None));
    assert!(refused(&client, "docs_list_trash", json!({"cursor": "zz"})).await.contains("cursor is not valid"));
    assert!(refused(&client, "docs_list_trash", json!({"space_id": "nowhere"})).await.contains("not found"));
}

#[tokio::test]
async fn the_tool_list_is_the_twelve_with_honest_annotations() {
    let Some(served) = served("docs_mcp_list").await else { return };
    let client = connect(&served.url, &format!("{}|docs.read", served.person.user_id), Default::default()).await;
    let tools = client.list_all_tools().await.unwrap();
    let hints = |name: &str| {
        let a = tools.iter().find(|t| t.name == name).unwrap().annotations.clone().unwrap();
        (a.read_only_hint.unwrap(), a.destructive_hint.unwrap(), a.idempotent_hint.unwrap())
    };
    assert_eq!(tools.len(), 12);
    for read in ["docs_search", "docs_list_tree", "docs_read", "docs_read_blocks", "docs_list_trash"] {
        assert_eq!(hints(read), (true, false, true), "{read}");
    }
    assert_eq!(hints("docs_create"), (false, false, false));
    for overwrite in ["docs_edit", "docs_replace"] {
        assert_eq!(hints(overwrite), (false, true, false), "{overwrite}");
    }
    for same in ["docs_update", "docs_move", "docs_trash"] {
        assert_eq!(hints(same), (false, true, true), "{same}");
    }
    assert_eq!(hints("docs_restore"), (false, false, true));
}
