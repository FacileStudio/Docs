//! Link previews for embeds: a pasted link unfurled from its Open Graph, Twitter and `<title>`
//! tags, through a client that reaches public `https` addresses only. A deliberate copy of
//! Toile's card unfurl, without its suite-app cards.

mod cache;
mod guard;
#[cfg(test)]
mod guard_tests;
mod og;
mod router;
mod unfurl;

use serde::Serialize;
use url::Url;

pub use cache::Previews;
pub use router::routes;

/// What `GET /embeds/resolve` answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Preview {
    /// The page title, or the host when the page says nothing.
    pub title: String,
    /// The page description.
    pub description: Option<String>,
    /// A preview image, `https` only.
    pub image: Option<String>,
    /// The name of the site.
    pub site: Option<String>,
    /// The site icon, `https` only.
    pub favicon: Option<String>,
}

impl Preview {
    /// A preview that only names the host of `url`.
    pub fn host(url: &Url) -> Self {
        Preview {
            title: url.host_str().unwrap_or(url.as_str()).to_owned(),
            description: None,
            image: None,
            site: None,
            favicon: None,
        }
    }
}
