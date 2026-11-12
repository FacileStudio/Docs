mod common;

use common::fixture;
use serde_json::json;

const LATE_CHILD: &str = "INSERT INTO documents (space_id, owner_id, parent_id, position, title, schema_version, \
     content, text, created_by, updated_by) SELECT space_id, owner_id, id, 1, 'Late', schema_version, content, '', \
     created_by, created_by FROM documents WHERE id = $1::uuid RETURNING id";

#[tokio::test]
async fn a_create_racing_a_trash_of_its_parent_is_refused() {
    let Some(pool) = common::pool("docs_trash_race_create").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let parent = fixture::id(&fixture::page(&app, &roles.owner, Some(&roles.space), "parent").await);
    let mut trash = pool.begin().await.unwrap();
    sqlx::query("UPDATE documents SET trashed_at = now() WHERE id = $1::uuid")
        .bind(&parent)
        .execute(&mut *trash)
        .await
        .unwrap();
    let child = json!({"title": "Child", "parent_id": parent});
    let (created, ()) = tokio::join!(app.call("POST", "/api/documents", Some(&roles.owner), Some(child)), async {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        trash.commit().await.unwrap();
    });
    assert_eq!(created.status, 412, "{}", created.text());
    let live: i64 =
        sqlx::query_scalar("SELECT count(*) FROM documents WHERE parent_id = $1::uuid AND trashed_at IS NULL")
            .bind(&parent)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(live, 0);
}

#[tokio::test]
async fn a_trash_racing_a_create_under_it_takes_the_new_child() {
    let Some(pool) = common::pool("docs_trash_race_trash").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let parent = fixture::id(&fixture::page(&app, &roles.owner, Some(&roles.space), "parent").await);
    let mut create = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM documents WHERE id = $1::uuid FOR SHARE")
        .bind(&parent)
        .execute(&mut *create)
        .await
        .unwrap();
    let child: uuid::Uuid = sqlx::query_scalar(LATE_CHILD).bind(&parent).fetch_one(&mut *create).await.unwrap();
    let path = format!("/api/documents/{parent}");
    let (trashed, ()) = tokio::join!(app.call("DELETE", &path, Some(&roles.owner), None), async {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        create.commit().await.unwrap();
    });
    assert_eq!(trashed.status, 204, "{}", trashed.text());
    let at: Option<time::OffsetDateTime> = sqlx::query_scalar("SELECT trashed_at FROM documents WHERE id = $1")
        .bind(child)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(at.is_some(), "the child created during the trash stayed live");
}
