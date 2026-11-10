mod common;

use common::fixture;
use serde_json::json;

#[tokio::test]
async fn a_member_of_another_space_cannot_list_create_or_search_in_it() {
    let Some(pool) = common::pool("docs_scopes").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    fixture::page(&app, &roles.owner, Some(&roles.space), "secret plans").await;
    let space = &roles.space;
    let outsider = Some(&roles.outsider);
    assert_eq!(app.call("GET", &format!("/api/documents?space_id={space}"), outsider, None).await.status, 404);
    assert_eq!(app.call("GET", &format!("/api/search?q=secret&space_id={space}"), outsider, None).await.status, 404);
    let create = json!({"title": "x", "space_id": space});
    assert_eq!(app.call("POST", "/api/documents", outsider, Some(create)).await.status, 404);
    for path in ["/api/search?q=secret".to_owned(), format!("/api/search?q=secret&space_id={}", roles.other_space)] {
        let hits = app.call("GET", &path, outsider, None).await;
        assert_eq!(hits.status, 200);
        assert_eq!(hits.json(), json!([]), "{path}");
    }
}

#[tokio::test]
async fn personal_pages_belong_to_their_owner_alone() {
    let Some(pool) = common::pool("docs_personal").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let doc = fixture::id(&fixture::page(&app, &roles.member, None, "diary").await);
    for who in [&roles.owner, &roles.admin, &roles.outsider] {
        assert_eq!(app.call("GET", &format!("/api/documents/{doc}"), Some(who), None).await.status, 404);
        assert_eq!(app.call("GET", "/api/search?q=diary", Some(who), None).await.json(), json!([]));
    }
    let mine = app.call("GET", "/api/search?q=diary", Some(&roles.member), None).await.json();
    assert_eq!(mine[0]["id"], json!(doc));
}

#[tokio::test]
async fn spaces_lists_only_the_callers_spaces_with_their_role() {
    let Some(pool) = common::pool("docs_spaces").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let answer = app.call("GET", "/api/spaces", Some(&roles.member), None).await.json();
    assert_eq!(answer, json!({"spaces": [{"id": roles.space, "name": roles.space, "role": "member"}]}));
    assert_eq!(app.call("GET", "/api/spaces", None, None).await.status, 401);
}
