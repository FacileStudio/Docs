mod common;

use common::http::with_cookie;
use common::mcp::{Served, args, served};
use serde_json::{Value, json};
use tronc::mcptest::{ClientOptions, call, connect, decode};

async fn version(app: &common::App, id: &str) -> i64 {
    sqlx::query_scalar("SELECT version FROM documents WHERE id = $1::uuid").bind(id).fetch_one(&app.pool).await.unwrap()
}

async fn pending_replace(served: &Served) -> (String, String) {
    let token = format!("{}|docs.read,docs.write", served.person.user_id);
    let client = connect(&served.url, &token, ClientOptions::default()).await;
    let created: Value = decode(&call(&client, "docs_create", args(json!({"title": "Own", "markdown": "a"}))).await);
    let id = created["id"].as_str().unwrap().to_owned();
    let ask = json!({"document_id": id, "markdown": "b", "expect_version": 1});
    let pending: Value = decode(&call(&client, "docs_replace", args(ask)).await);
    let action = pending["confirm_url"].as_str().unwrap().rsplit('=').next().unwrap().to_owned();
    (id, format!("/api/mcp/pending/{action}/confirm"))
}

async fn dash_cookie(app: &common::App, user_id: i64) -> String {
    let token = format!("b-{}", porte::new_token().unwrap());
    let row = porte::Session {
        user_id,
        token_hash: porte::hash_token(&token),
        expires_at: Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
        ..porte::Session::default()
    };
    app.kits.sessions.store().create(row).await.unwrap();
    token
}

async fn refused_cookies(served: &Served) -> Vec<(&'static str, String)> {
    let app = &served.app;
    let (named, _) = app.kits.sessions.issue(served.person.user_id, "agent").await.unwrap();
    vec![
        ("a CLI exchange token", served.person.token.clone()),
        ("a labelled API token", named),
        ("a b- cookie", dash_cookie(app, served.person.user_id).await),
    ]
}

#[tokio::test]
async fn only_a_browser_login_confirms_on_the_page() {
    let Some(served) = served("docs_mcp_own_session").await else { return };
    let (id, path) = pending_replace(&served).await;
    let router = &served.app.router;
    for (name, cookie) in refused_cookies(&served).await {
        for (method, route) in [("POST", path.as_str()), ("GET", "/api/mcp/pending")] {
            let refused = with_cookie(router, method, route, &cookie).await;
            assert_eq!(refused.status, 403, "{name} {method}: {}", refused.text());
            assert!(refused.text().contains("permission_denied"), "{name}: {}", refused.text());
        }
    }
    let bearer = served.app.call("POST", &path, Some(&served.person), None).await;
    assert_eq!((bearer.status.as_u16(), version(&served.app, &id).await), (403, 1), "{}", bearer.text());
    let login = common::http::browser_login(&served.app.kits.sessions, served.person.user_id).await;
    let done = with_cookie(router, "POST", &path, &login).await;
    assert_eq!(done.json(), json!({"status": "done"}), "{}", done.text());
    assert_eq!(version(&served.app, &id).await, 2);
}
