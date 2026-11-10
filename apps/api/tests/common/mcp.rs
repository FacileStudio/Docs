use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use http::request::Parts;
use serde_json::Value;
use tronc::mcp::{Options, TokenInfo, Verifier, VerifyError, VerifyFuture};

use super::{App, Person};

struct Tokens;

impl Verifier for Tokens {
    fn verify<'a>(&'a self, token: &'a str, _request: &'a Parts) -> VerifyFuture<'a> {
        Box::pin(async move {
            let (user, scopes) = token.split_once('|').ok_or(VerifyError::Invalid)?;
            Ok(TokenInfo {
                user_id: user.to_owned(),
                scopes: scopes.split(',').map(str::to_owned).collect(),
                expires_at: Some(SystemTime::now() + Duration::from_secs(600)),
                extra: HashMap::new(),
            })
        })
    }
}

pub struct Served {
    pub app: App,
    pub url: String,
    pub person: Person,
}

pub async fn served(prefix: &str) -> Option<Served> {
    let pool = super::pool(prefix).await?;
    let mut app = super::app(pool).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    app.state.confirmations = Some(api::mcp::confirmations(&app.state, &url, b"test-state-key").unwrap());
    let options = Options::new("docs", url.clone(), "https://sso.test", Arc::new(Tokens));
    app.router = api::router::build_router(app.state.clone(), &app.kits, Some(options)).unwrap();
    let router = app.router.clone().into_make_service_with_connect_info::<SocketAddr>();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let person = app.person("agent-owner").await;
    Some(Served { app, url, person })
}

pub fn args(value: Value) -> tronc::mcp::rmcp::model::JsonObject {
    serde_json::from_value(value).unwrap()
}
