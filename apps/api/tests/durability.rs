mod common;

use std::process::{Child, Command, Stdio};
use std::time::Duration;

use common::typing::{read, type_into_first_paragraph};
use common::ysync::Peer;
use sqlx::PgPool;

struct Server {
    child: Child,
    port: u16,
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

async fn start(database: &str, port: u16) -> Server {
    let child = Command::new(env!("CARGO_BIN_EXE_api"))
        .env("DATABASE_URL", database)
        .env("PORT", port.to_string())
        .env("DATA_DIR", common::data_dir())
        .env("LOG_LEVEL", "warn")
        .env_remove("OIDC_ISSUER")
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    for _ in 0..600 {
        if reqwest::get(format!("http://127.0.0.1:{port}/health")).await.is_ok() {
            return Server { child, port };
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let mut child = child;
    child.kill().unwrap();
    child.wait().unwrap();
    panic!("the api did not start");
}

impl Server {
    fn kill(mut self) -> u16 {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
        self.port
    }
}

async fn drop_database(url: &str) {
    let admin = PgPool::connect(&tronc::testdb::url().unwrap()).await.unwrap();
    let name = url.rsplit('/').next().unwrap().split('?').next().unwrap();
    sqlx::query(sqlx::AssertSqlSafe(format!("DROP DATABASE {name} WITH (FORCE)"))).execute(&admin).await.unwrap();
}

async fn database() -> Option<(String, PgPool)> {
    let admin = tronc::testdb::url()?;
    let name = format!("docs_kill_{}", uuid::Uuid::new_v4().simple());
    let pool = PgPool::connect(&admin).await.ok()?;
    sqlx::query(sqlx::AssertSqlSafe(format!("CREATE DATABASE {name}"))).execute(&pool).await.ok()?;
    let (base, query) = admin.split_once('?').map_or((admin.as_str(), ""), |(b, q)| (b, q));
    let url = format!("{}/{name}?{query}", base.rsplit_once('/').unwrap().0);
    Some((url.clone(), PgPool::connect(&url).await.ok()?))
}

async fn session(pool: &PgPool) -> String {
    let user: i64 = sqlx::query_scalar("INSERT INTO porte_users (email) VALUES ('kill@example.test') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let token = porte::new_token().unwrap();
    sqlx::query("INSERT INTO porte_sessions (token_hash, user_id) VALUES ($1, $2)")
        .bind(porte::hash_token(&token))
        .bind(user)
        .execute(pool)
        .await
        .unwrap();
    token
}

async fn page(port: u16, token: &str) -> String {
    let answer: serde_json::Value = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{port}/api/documents"))
        .bearer_auth(token)
        .json(&serde_json::json!({"title": "Durable", "markdown": "start"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    answer["id"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn killing_the_process_loses_no_edit_the_client_holds_or_the_server_saved() {
    let Some((url, pool)) = database().await else { return };
    let server = start(&url, free_port()).await;
    let token = session(&pool).await;
    let id = page(server.port, &token).await;
    let ws = |port: u16| format!("ws://127.0.0.1:{port}/api/documents/{id}/ws");
    let mut peer = Peer::new();
    peer.connect(&ws(server.port), &token).await;
    peer.edit(|txn, f| type_into_first_paragraph(txn, f, 5, " unsaved")).await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let server = start(&url, server.kill()).await;
    peer.disconnect();
    peer.connect(&ws(server.port), &token).await;
    peer.edit(|txn, f| type_into_first_paragraph(txn, f, 100, " saved")).await;
    tokio::time::sleep(Duration::from_secs(3)).await;
    let server = start(&url, server.kill()).await;
    let mut fresh = Peer::new();
    fresh.connect(&ws(server.port), &token).await;
    let text = encre::markdown::plain_text(&read(&fresh.doc));
    server.kill();
    pool.close().await;
    drop_database(&url).await;
    assert_eq!(text, "start unsaved saved");
}
