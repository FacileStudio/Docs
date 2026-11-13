use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::Preview;

/// How long a resolved preview is reused for the same URL.
pub const TTL: Duration = Duration::from_secs(300);
const MAX_ENTRIES: usize = 10_000;

type Entries = HashMap<String, (Instant, Preview)>;

/// Resolved previews per URL for five minutes, and the guarded unfurl client.
#[derive(Clone)]
pub struct Previews {
    entries: Arc<Mutex<Entries>>,
    /// The SSRF-guarded client.
    pub client: reqwest::Client,
}

impl Default for Previews {
    fn default() -> Self {
        Previews { entries: Arc::default(), client: super::guard::client() }
    }
}

impl Previews {
    fn lock(&self) -> std::sync::MutexGuard<'_, Entries> {
        self.entries.lock().unwrap_or_else(|poison| poison.into_inner())
    }

    /// The preview cached for `url`, while it is fresh.
    pub fn get(&self, url: &str) -> Option<Preview> {
        let entries = self.lock();
        let (at, preview) = entries.get(url)?;
        (at.elapsed() < TTL).then(|| preview.clone())
    }

    /// Keeps `preview` for `url`; past 10,000 entries the stale ones go first.
    pub fn put(&self, url: &str, preview: Preview) {
        let mut entries = self.lock();
        if entries.len() >= MAX_ENTRIES {
            entries.retain(|_, (at, _)| at.elapsed() < TTL);
        }
        if entries.len() >= MAX_ENTRIES {
            entries.clear();
        }
        entries.insert(url.to_owned(), (Instant::now(), preview));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_preview_is_reused_for_the_same_url_only() {
        let previews = Previews::default();
        let url = url::Url::parse("https://example.com/a").unwrap();
        previews.put(url.as_str(), Preview::host(&url));
        assert_eq!(previews.get("https://example.com/a").unwrap().title, "example.com");
        assert!(previews.get("https://example.com/b").is_none());
    }
}
