mod common;

use std::time::Duration;

use common::fixture;
use common::typing::{read, type_into_first_paragraph};
use common::ysync::Peer;
use serde_json::json;

async fn stored(app: &common::App, id: &str) -> serde_json::Value {
    sqlx::query_scalar::<_, sqlx::types::Json<serde_json::Value>>("SELECT content FROM documents WHERE id = $1::uuid")
        .bind(id)
        .fetch_one(&app.pool)
        .await
        .unwrap()
        .0
}

#[tokio::test]
async fn two_peers_editing_one_paragraph_offline_converge_and_the_store_matches() {
    let Some(pool) = common::pool("docs_rooms_converge").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::page(&app, &roles.member, Some(&roles.space), "hello world").await);
    let url = format!("{}/api/documents/{id}/ws", app.serve().await);
    let (mut a, mut b) = (Peer::new(), Peer::new());
    a.connect(&url, &roles.member.token).await;
    b.connect(&url, &roles.owner.token).await;
    assert_eq!(read(&a.doc), read(&b.doc));
    a.disconnect();
    b.disconnect();
    for i in 0..10 {
        a.edit(|txn, f| type_into_first_paragraph(txn, f, 0, &format!("a{i} "))).await;
        b.edit(|txn, f| type_into_first_paragraph(txn, f, 100, &format!(" b{i}"))).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    a.connect(&url, &roles.member.token).await;
    b.connect(&url, &roles.owner.token).await;
    a.pump(Duration::from_millis(500)).await;
    b.pump(Duration::from_millis(500)).await;
    let (left, right) = (read(&a.doc), read(&b.doc));
    assert_eq!(left, right);
    let text = encre::markdown::plain_text(&left);
    assert!(text.contains("a9 ") && text.contains(" b9") && text.contains("hello world"), "{text}");
    a.disconnect();
    b.disconnect();
    tokio::time::sleep(Duration::from_millis(800)).await;
    assert_eq!(stored(&app, &id).await, serde_json::to_value(&left).unwrap());
}

#[tokio::test]
async fn a_schema_breaking_update_is_reverted_and_its_sender_closed_with_1008() {
    let Some(pool) = common::pool("docs_rooms_schema").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::page(&app, &roles.member, None, "safe").await);
    let url = format!("{}/api/documents/{id}/ws", app.serve().await);
    let mut bad = Peer::new();
    bad.connect(&url, &roles.member.token).await;
    let before = read(&bad.doc);
    bad.edit(|txn, fragment| {
        use yrs::{Xml, XmlElementPrelim, XmlFragment};
        let heading = fragment.push_back(txn, XmlElementPrelim::empty("heading"));
        heading.insert_attribute(txn, "level", "9");
    })
    .await;
    bad.pump(Duration::from_millis(800)).await;
    assert_eq!(bad.closed, Some(1008));
    let mut fresh = Peer::new();
    fresh.connect(&url, &roles.member.token).await;
    assert_eq!(read(&fresh.doc), before);
    fresh.edit(|txn, f| type_into_first_paragraph(txn, f, 4, "!")).await;
    fresh.pump(Duration::from_millis(300)).await;
    assert_eq!(fresh.closed, None);
}

#[tokio::test]
async fn a_ten_thousand_deep_update_is_refused_with_1008_and_the_room_survives() {
    let Some(pool) = common::pool("docs_rooms_deep").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::page(&app, &roles.member, None, "safe").await);
    let url = format!("{}/api/documents/{id}/ws", app.serve().await);
    let mut hostile = Peer::new();
    hostile.connect(&url, &roles.member.token).await;
    let before = read(&hostile.doc);
    hostile
        .edit(|txn, fragment| {
            use yrs::{XmlElementPrelim, XmlFragment};
            let mut innermost = fragment.push_back(txn, XmlElementPrelim::empty("callout"));
            for _ in 0..10_000 {
                innermost = innermost.push_back(txn, XmlElementPrelim::empty("callout"));
            }
        })
        .await;
    hostile.pump(Duration::from_millis(1500)).await;
    assert_eq!(hostile.closed, Some(1008));
    let mut fresh = Peer::new();
    fresh.connect(&url, &roles.member.token).await;
    assert_eq!(read(&fresh.doc), before);
    fresh.edit(|txn, f| type_into_first_paragraph(txn, f, 4, "!")).await;
    fresh.pump(Duration::from_millis(300)).await;
    assert_eq!(fresh.closed, None);
    assert_eq!(app.call("GET", &format!("/api/documents/{id}"), Some(&roles.member), None).await.status, 200);
}

#[tokio::test]
async fn a_viewer_update_is_dropped_and_rest_ops_reach_open_editors() {
    let Some(pool) = common::pool("docs_rooms_views").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::page(&app, &roles.member, None, "keep").await);
    let url = format!("{}/api/documents/{id}/ws", app.serve().await);
    let mut editor = Peer::new();
    editor.connect(&url, &roles.member.token).await;
    let ops = json!({"ops": [{"op": "append", "markdown": "from an agent"}]});
    let answer = app.call("POST", &format!("/api/documents/{id}/ops"), Some(&roles.member), Some(ops)).await;
    assert_eq!(answer.status, 200, "{}", answer.text());
    editor.pump(Duration::from_millis(500)).await;
    assert!(encre::markdown::plain_text(&read(&editor.doc)).contains("from an agent"));
    editor.disconnect();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(app.call("DELETE", &format!("/api/documents/{id}"), Some(&roles.member), None).await.status, 204);
    let mut viewer = Peer::new();
    viewer.connect(&url, &roles.member.token).await;
    let before = read(&viewer.doc);
    viewer.edit(|txn, f| type_into_first_paragraph(txn, f, 0, "nope ")).await;
    viewer.pump(Duration::from_millis(300)).await;
    let mut other = Peer::new();
    other.connect(&url, &roles.member.token).await;
    assert_eq!(read(&other.doc), before);
    assert_eq!(app.call("GET", &url.replace("ws://", "http://"), Some(&roles.outsider), None).await.status, 404);
}

#[tokio::test]
async fn a_cookie_upgrade_from_another_origin_is_refused() {
    let Some(pool) = common::pool("docs_rooms_origin").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::page(&app, &roles.member, None, "x").await);
    let url = format!("{}/api/documents/{id}/ws", app.serve().await);
    let attempt = |origin: &'static str| {
        use tokio_tungstenite::tungstenite::client::IntoClientRequest;
        let mut request = url.as_str().into_client_request().unwrap();
        request.headers_mut().insert("cookie", format!("session={}", roles.member.token).parse().unwrap());
        request.headers_mut().insert("origin", origin.parse().unwrap());
        tokio_tungstenite::connect_async(request)
    };
    match attempt("https://evil.test").await {
        Err(tokio_tungstenite::tungstenite::Error::Http(response)) => assert_eq!(response.status(), 403),
        other => panic!("cross-origin upgrade not refused: {:?}", other.map(|(_, r)| r.status())),
    }
}

#[tokio::test]
async fn a_frame_over_two_mebibytes_closes_with_1009() {
    let Some(pool) = common::pool("docs_rooms_size").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::page(&app, &roles.member, None, "x").await);
    let url = format!("{}/api/documents/{id}/ws", app.serve().await);
    let mut peer = Peer::new();
    peer.connect(&url, &roles.member.token).await;
    peer.send_raw(vec![0u8; (2 << 20) + 1]).await;
    peer.pump(Duration::from_secs(2)).await;
    assert_eq!(peer.closed, Some(1009));
}

#[tokio::test]
async fn a_space_removed_by_the_directory_closes_its_sockets_and_refuses_reconnects() {
    let Some(pool) = common::pool("docs_rooms_retire").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::page(&app, &roles.member, Some(&roles.space), "shared").await);
    let url = format!("{}/api/documents/{id}/ws", app.serve().await);
    let mut peer = Peer::new();
    peer.connect(&url, &roles.member.token).await;
    let apply = api::auth::SpacesApply::new(pool, app.state.hub.clone(), common::ISSUER.to_owned());
    let version = time::OffsetDateTime::now_utc().unix_timestamp_nanos() as i64;
    let other = porte::directory::Space { id: "elsewhere".into(), ..Default::default() };
    let snapshot = porte::directory::Snapshot { version, spaces: vec![other], ..Default::default() };
    assert!(apply.apply_snapshot(&snapshot).await.unwrap());
    peer.pump(Duration::from_secs(1)).await;
    assert!(peer.closed.is_some(), "the socket stayed open");
    let mut request =
        tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(url.as_str()).unwrap();
    request.headers_mut().insert("authorization", format!("Bearer {}", roles.member.token).parse().unwrap());
    assert!(tokio_tungstenite::connect_async(request).await.is_err());
}
