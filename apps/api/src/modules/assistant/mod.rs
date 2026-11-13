//! `GET /assistant`: the name and accent of the person's Bulle assistant, read from Bulle with a
//! token exchanged for them. Any failure answers the default, so the route never refuses.

mod router;

use std::time::Duration;

use http::Method;
use porte::exchange::{Exchanger, Target};
use serde::{Deserialize, Serialize};
use tronc::bulle::{self, ForgetFuture, Request, TokenFuture, TokenSource};
use tronc::env::Vars;

pub use router::routes;

use crate::state::AppState;

/// The name shown when Bulle has none to give.
pub const DEFAULT_NAME: &str = "Bulle";

/// How long Docs waits for Bulle.
pub const TIMEOUT: Duration = Duration::from_secs(5);

/// Where the suite's assistant lives and the audience of the tokens exchanged for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bulle {
    /// `BULLE_URL`, without a trailing slash.
    pub url: String,
    /// `BULLE_CLIENT_ID`: Bulle's Registre client id.
    pub audience: String,
}

impl Bulle {
    /// Reads `BULLE_URL` (default `https://bulle.facile.studio`) and `BULLE_CLIENT_ID` (default
    /// `bulle`).
    pub fn from_vars(vars: &Vars) -> Self {
        let url = vars.string("BULLE_URL", "https://bulle.facile.studio").trim().trim_end_matches('/').to_owned();
        Bulle { url, audience: vars.string("BULLE_CLIENT_ID", "bulle").trim().to_owned() }
    }
}

/// What `GET /assistant` answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Assistant {
    /// The assistant's name, "Bulle" by default.
    pub name: String,
    /// The assistant's color, when Bulle gives one.
    pub accent: Option<String>,
}

impl Default for Assistant {
    fn default() -> Self {
        Assistant { name: DEFAULT_NAME.to_owned(), accent: None }
    }
}

#[derive(Deserialize)]
struct Profile {
    name: Option<String>,
    accent: Option<String>,
}

struct Tokens<'a> {
    exchanger: &'a Exchanger,
    user_id: i64,
    target: Target,
}

impl TokenSource for Tokens<'_> {
    fn token(&self) -> TokenFuture<'_> {
        Box::pin(async move { Ok(self.exchanger.token(self.user_id, &self.target).await?) })
    }

    fn forget(&self) -> Option<ForgetFuture<'_>> {
        self.exchanger.forget(self.user_id, &self.target);
        Some(Box::pin(std::future::ready(())))
    }
}

fn trimmed(raw: Option<String>) -> Option<String> {
    raw.map(|value| value.trim().chars().take(64).collect::<String>()).filter(|value| !value.is_empty())
}

/// The assistant Bulle's `GET /api/assistant` describes with the token `tokens` gives, or the
/// default on any failure.
pub async fn ask<S: TokenSource + ?Sized>(client: &bulle::Client, tokens: &S) -> Assistant {
    match client.call::<Profile, S>(tokens, Request::new(Method::GET, ["assistant"])).await {
        Ok(profile) => Assistant {
            name: trimmed(profile.name).unwrap_or_else(|| DEFAULT_NAME.to_owned()),
            accent: trimmed(profile.accent),
        },
        Err(error) => {
            tracing::debug!(%error, "assistant: bulle answered no profile");
            Assistant::default()
        }
    }
}

/// The client Docs reaches Bulle at `url` with: no redirects, no proxy, [`TIMEOUT`].
pub fn client(url: &str) -> Option<bulle::Client> {
    let http = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()
        .ok()?;
    bulle::Client::with_http(url, http).ok()
}

/// `user_id`'s assistant, or the default while Registre or Bulle is not configured.
pub async fn assistant(state: &AppState, user_id: i64) -> Assistant {
    let Some(exchanger) = state.exchanger.as_ref().filter(|_| !state.bulle.audience.is_empty()) else {
        return Assistant::default();
    };
    let Some(client) = client(&state.bulle.url) else { return Assistant::default() };
    ask(&client, &Tokens { exchanger, user_id, target: Target::new(&state.bulle.audience, ["bulle.read"]) }).await
}
