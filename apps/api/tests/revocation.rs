mod common;

use std::time::Duration;

use api::auth::AppData;
use common::fixture;
use common::typing::{read, type_into_first_paragraph};
use common::ysync::Peer;
use porte::directory::PersonDeleted;

async fn stored(pool: &sqlx::PgPool, id: &str) -> Option<serde_json::Value> {
    let row: Option<sqlx::types::Json<serde_json::Value>> =
        sqlx::query_scalar("SELECT content FROM documents WHERE id = $1::uuid")
            .bind(id)
            .fetch_optional(pool)
            .await
            .unwrap();
    row.map(|json| json.0)
}

#[tokio::test]
async fn a_socket_closes_with_1008_once_its_session_is_logged_out() {
    let Some(pool) = common::pool("docs_revoke_logout").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::page(&app, &roles.member, Some(&roles.space), "open").await);
    let url = format!("{}/api/documents/{id}/ws", app.serve().await);
    let (mut gone, mut stays) = (Peer::new(), Peer::new());
    gone.connect(&url, &roles.member.token).await;
    stays.connect(&url, &roles.owner.token).await;
    sqlx::query("DELETE FROM porte_sessions WHERE user_id = $1")
        .bind(roles.member.user_id)
        .execute(&app.pool)
        .await
        .unwrap();
    gone.pump(Duration::from_millis(800)).await;
    stays.pump(Duration::from_millis(300)).await;
    assert_eq!(gone.closed, Some(1008));
    assert_eq!(stays.closed, None);
}

#[tokio::test]
async fn erasing_a_person_closes_their_sockets_and_still_saves_their_last_shared_edit() {
    let Some(pool) = common::pool("docs_revoke_erase").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let personal = fixture::id(&fixture::page(&app, &roles.member, None, "mine").await);
    let shared = fixture::id(&fixture::page(&app, &roles.member, Some(&roles.space), "ours").await);
    let base = app.serve().await;
    let (mut own, mut erased, mut other) = (Peer::new(), Peer::new(), Peer::new());
    own.connect(&format!("{base}/api/documents/{personal}/ws"), &roles.member.token).await;
    erased.connect(&format!("{base}/api/documents/{shared}/ws"), &roles.member.token).await;
    other.connect(&format!("{base}/api/documents/{shared}/ws"), &roles.owner.token).await;
    own.edit(|txn, f| type_into_first_paragraph(txn, f, 0, "secret ")).await;
    erased.edit(|txn, f| type_into_first_paragraph(txn, f, 0, "x")).await;
    erased.pump(Duration::from_millis(200)).await;
    AppData::new(pool.clone(), app.state.hub.clone())
        .on_person_deleted(common::ISSUER, &roles.member.subject)
        .await
        .unwrap();
    own.pump(Duration::from_millis(500)).await;
    erased.pump(Duration::from_millis(500)).await;
    assert_eq!((own.closed, erased.closed), (Some(1008), Some(1008)));
    other.pump(Duration::from_millis(2800)).await;
    assert_eq!(other.closed, None);
    assert_eq!(stored(&pool, &shared).await, Some(serde_json::to_value(read(&other.doc)).unwrap()));
    assert_eq!(stored(&pool, &personal).await, None);
}
