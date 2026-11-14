//! `GET /assistant`: the name and accent of the person's Bulle assistant, read from Bulle with a
//! token exchanged for them. Any failure answers the default, so the route never refuses.

mod router;

use std::time::Duration;

use http::Method;
use porte::exchange::Target;
use serde::{Deserialize, Serialize};
use tronc::bulle::{self, Request, TokenSource};
use tronc::env::{EnvError, Vars};

pub use router::routes;

use crate::state::AppState;

/// The name shown when Bulle has none to give.
pub const DEFAULT_NAME: &str = "Bulle";

/// How long Docs waits for Bulle.
pub const TIMEOUT: Duration = Duration::from_secs(5);

/// The Bulle Docs frames and asks when `BULLE_URL` is unset.
pub const DEFAULT_URL: &str = "https://bulle.facile.studio";

/// Where the suite's assistant lives and the audience of the tokens exchanged for it.
#[derive(Debug, Clone)]
pub struct Bulle {
    /// `BULLE_URL`: an http(s) origin without a trailing slash. The client frames its chat from
    /// it and the CSP allows it in `frame-src`.
    pub url: String,
    /// `BULLE_CLIENT_ID`: Bulle's Registre client id.
    pub audience: String,
    /// The pooled client for `url`, built once at boot.
    pub client: Option<bulle::Client>,
}

impl Bulle {
    /// Reads `BULLE_URL` (default [`DEFAULT_URL`]), refusing anything but an http(s) origin, and
    /// `BULLE_CLIENT_ID` (default `bulle`).
    pub fn from_vars(vars: &Vars) -> Result<Self, EnvError> {
        let url = origin(&vars.string("BULLE_URL", "")).map_err(EnvError)?;
        let client = client(&url);
        Ok(Bulle { url, audience: vars.string("BULLE_CLIENT_ID", "bulle").trim().to_owned(), client })
    }
}

/// The origin in `raw` without a trailing slash: an http(s) URL with a host and no path, query,
/// fragment or credentials, or [`DEFAULT_URL`] when `raw` is blank.
pub fn origin(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Ok(DEFAULT_URL.to_owned());
    }
    let refused = || format!("BULLE_URL {raw:?} must be an http(s) origin, without a path");
    let url = url::Url::parse(trimmed).map_err(|_| refused())?;
    let bare = matches!(url.scheme(), "https" | "http")
        && url.host_str().is_some_and(|host| !host.is_empty())
        && url.username().is_empty()
        && url.password().is_none()
        && url.path() == "/"
        && url.query().is_none()
        && url.fragment().is_none();
    bare.then(|| url.origin().ascii_serialization()).ok_or_else(refused)
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

/// tronc's Bulle client at `url`: pooled, never following a redirect, [`TIMEOUT`].
pub fn client(url: &str) -> Option<bulle::Client> {
    bulle::Client::with_timeout(url, TIMEOUT).ok()
}

/// `user_id`'s assistant, or the default while Registre or Bulle is not configured.
pub async fn assistant(state: &AppState, user_id: i64) -> Assistant {
    let Some(exchanger) = state.exchanger.as_ref().filter(|_| !state.bulle.audience.is_empty()) else {
        return Assistant::default();
    };
    let Some(client) = state.bulle.client.as_ref() else { return Assistant::default() };
    ask(client, &exchanger.bound(user_id, Target::new(&state.bulle.audience, ["bulle.read"]))).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bulle_is_an_http_origin_with_the_suite_default() {
        assert_eq!(origin(" ").as_deref(), Ok(DEFAULT_URL));
        assert_eq!(origin("https://Bulle.test/").as_deref(), Ok("https://bulle.test"));
        assert_eq!(origin("http://localhost:5174").as_deref(), Ok("http://localhost:5174"));
        for bad in [
            "javascript:alert(1)",
            "https://bulle.test/embed",
            "bulle.test",
            "https://bulle.test?x=1",
            "https://a@bulle.test",
        ] {
            assert!(origin(bad).is_err(), "{bad}");
        }
    }
}
