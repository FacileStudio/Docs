mod common;

use api::auth::Pictures;
use porte::Claims;

#[tokio::test]
async fn a_sign_in_keeps_the_registre_picture_as_the_avatar() {
    let Some(pool) = common::pool("docs_avatar").await else { return };
    let app = common::app(pool.clone()).await;
    let users = Pictures::wrap(porte::pg::Store::new(pool).users());
    let picture = "https://sso.test/avatars/ada";
    let claims = Claims {
        provider: common::ISSUER.into(),
        subject: "ada".into(),
        email: "ada@example.test".into(),
        email_verified: true,
        name: "Ada".into(),
        picture: picture.into(),
        ..Claims::default()
    };
    let user_id = users.upsert_from_oidc(&claims).await.unwrap();
    let (token, _) = app.kits.sessions.issue(user_id, "test").await.unwrap();
    let person = common::Person { user_id, subject: "ada".into(), token };
    let me = app.call("GET", "/api/auth/me", Some(&person), None).await;
    assert_eq!(me.json()["avatar_url"], picture);

    let plain =
        Claims { subject: "bob".into(), email: "bob@example.test".into(), picture: "http://x/b".into(), ..claims };
    let bob = users.upsert_from_oidc(&plain).await.unwrap();
    let stored: String = sqlx::query_scalar("SELECT avatar_url FROM porte_users WHERE id = $1")
        .bind(bob)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(stored, "");
}

#[tokio::test]
async fn a_member_signed_in_before_gets_the_directory_picture() {
    let Some(pool) = common::pool("docs_avatar_member").await else { return };
    let app = common::app(pool).await;
    let person = app.person("carol").await;
    let session = app.call("GET", "/api/auth/session", Some(&person), None).await;
    assert!(session.json()["user"].get("avatar_url").is_none());
    let space = app.space(&[(&person, "member")]).await;
    sqlx::query("UPDATE space_members SET picture = 'https://sso.test/avatars/carol' WHERE space_id = $1")
        .bind(&space)
        .execute(&app.pool)
        .await
        .unwrap();
    let session = app.call("GET", "/api/auth/session", Some(&person), None).await;
    assert_eq!(session.json()["user"]["avatar_url"], "https://sso.test/avatars/carol");
}
