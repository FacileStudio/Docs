mod common;

use api::auth::SpacesApply;
use common::directory::{base, snapshot, space};
use common::fixture;
use serde_json::json;

#[tokio::test]
async fn a_space_missing_from_a_snapshot_is_retired_and_its_pages_trashed_until_it_returns() {
    let Some(pool) = common::pool("docs_dir_retire").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let doc = fixture::id(&fixture::page(&app, &roles.owner, Some(&roles.space), "kept safe").await);
    let apply = SpacesApply::new(pool.clone(), app.state.hub.clone());
    let version = base();
    assert!(apply.apply_snapshot(&snapshot(version, vec![space("other", &[])])).await.unwrap());
    let trashed: Option<time::OffsetDateTime> =
        sqlx::query_scalar("SELECT trashed_at FROM documents WHERE id = $1::uuid")
            .bind(&doc)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(trashed.is_some());
    for who in [&roles.owner, &roles.member] {
        assert_eq!(app.call("GET", &format!("/api/documents/{doc}"), Some(who), None).await.status, 404);
    }
    assert_eq!(app.call("GET", "/api/spaces", Some(&roles.owner), None).await.json(), json!({"spaces": []}));
    let back = space(&roles.space, &[(&roles.owner.subject, "owner")]);
    assert!(apply.apply_snapshot(&snapshot(version + 1, vec![back])).await.unwrap());
    let page = app.call("GET", &format!("/api/documents/{doc}"), Some(&roles.owner), None).await.json();
    assert!(page["trashed_at"].is_string(), "{page}");
    let restored = app.call("POST", &format!("/api/documents/{doc}/restore"), Some(&roles.owner), None).await;
    assert_eq!(restored.status, 200);
}

#[tokio::test]
async fn an_empty_snapshot_is_refused_while_spaces_exist() {
    let Some(pool) = common::pool("docs_dir_empty").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let apply = SpacesApply::new(pool.clone(), app.state.hub.clone());
    assert!(apply.apply_snapshot(&snapshot(base(), vec![])).await.is_err());
    let live: Option<time::OffsetDateTime> = sqlx::query_scalar("SELECT deleted_at FROM spaces WHERE id = $1")
        .bind(&roles.space)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(live.is_none());
    assert_eq!(
        app.call("GET", "/api/spaces", Some(&roles.member), None).await.json()["spaces"][0]["id"],
        json!(roles.space)
    );
}

#[tokio::test]
async fn the_purge_deletes_only_pages_trashed_more_than_thirty_days_ago() {
    let Some(pool) = common::pool("docs_purge").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let old = fixture::id(&fixture::page(&app, &roles.member, None, "old").await);
    let recent = fixture::id(&fixture::page(&app, &roles.member, None, "recent").await);
    for (id, age) in [(&old, "31 days"), (&recent, "1 day")] {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "UPDATE documents SET trashed_at = now() - interval '{age}' WHERE id = $1::uuid"
        )))
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    }
    assert!(api::modules::documents::purge::purge(&app.state).await.unwrap() >= 1);
    assert_eq!(app.call("GET", &format!("/api/documents/{old}"), Some(&roles.member), None).await.status, 404);
    assert_eq!(app.call("GET", &format!("/api/documents/{recent}"), Some(&roles.member), None).await.status, 200);
}
