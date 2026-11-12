mod common;

use axum::Json;
use axum::extract::State;
use axum::routing::get;
use porte::jwt::{Claims, Profile};
use porte::oidc::ProvisionError;
use serde_json::{Value, json};

async fn discovery(State(issuer): State<String>) -> Json<Value> {
    Json(json!({
        "issuer": issuer,
        "authorization_endpoint": format!("{issuer}/authorize"),
        "token_endpoint": format!("{issuer}/token"),
        "userinfo_endpoint": format!("{issuer}/userinfo"),
        "jwks_uri": format!("{issuer}/keys"),
    }))
}

async fn registre(subject: &str) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let issuer = format!("http://{}", listener.local_addr().unwrap());
    let deleted = json!({"deleted": [{"sub": subject, "deleted_at": "2026-11-12T09:00:00Z"}]});
    let router = axum::Router::new()
        .route("/.well-known/openid-configuration", get(discovery))
        .route("/keys", get(|| async { Json(json!({"keys": []})) }))
        .fallback(move || {
            let deleted = deleted.clone();
            async move { Json(deleted) }
        })
        .with_state(issuer.clone());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    issuer
}

async fn account(app: &common::App, issuer: &str, subject: &str) {
    let user: i64 = sqlx::query_scalar("INSERT INTO porte_users (email, name) VALUES ($1, 'Ada') RETURNING id")
        .bind(format!("{subject}@example.test"))
        .fetch_one(&app.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO porte_identities (user_id, provider, subject) VALUES ($1, $2, $3)")
        .bind(user)
        .bind(issuer)
        .bind(subject)
        .execute(&app.pool)
        .await
        .unwrap();
}

async fn users(app: &common::App, subject: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM porte_users WHERE email = $1")
        .bind(format!("{subject}@example.test"))
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

fn bearer(subject: &str) -> Claims {
    Claims {
        subject: subject.into(),
        email: format!("{subject}@example.test"),
        name: "Ada".into(),
        roles: None,
        scopes: vec!["docs.read".into()],
        actor: String::new(),
        audience: vec!["docs".into()],
        client_id: "claude".into(),
        id: String::new(),
        expires_at: time::OffsetDateTime::now_utc(),
        profile: Profile { email_verified: Some(true), ..Profile::default() },
    }
}

#[tokio::test]
async fn an_erased_subject_cannot_come_back() {
    let Some(pool) = common::pool("docs_erased_hold").await else { return };
    let subject = format!("erased-{}", uuid::Uuid::new_v4());
    let issuer = registre(&subject).await;
    let vars = [
        ("OIDC_ISSUER", issuer.as_str()),
        ("OIDC_CLIENT_ID", "docs"),
        ("OIDC_CLIENT_SECRET", "secret"),
        ("OIDC_REDIRECT_URL", "http://docs.test/api/auth/oidc/callback"),
        ("OIDC_SUCCESS_URL", "http://docs.test/"),
    ];
    let app = common::app_with(pool, &vars).await;
    let deletions = app.kits.deletions.clone().expect("the deletion loop is wired");
    account(&app, &issuer, &subject).await;
    deletions.poll().await.unwrap();
    assert_eq!(users(&app, &subject).await, 0);
    assert!(matches!(app.kits.oidc.provision_bearer(&bearer(&subject)).await, Err(ProvisionError::Erased)));
    assert_eq!(users(&app, &subject).await, 0, "a bearer call re-created the erased account");
    account(&app, &issuer, &subject).await;
    deletions.poll().await.unwrap();
    assert_eq!(users(&app, &subject).await, 0, "an account that raced the erasure survived the next poll");
}
