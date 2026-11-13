use url::Url;

use super::Preview;
use super::guard::allowed;
use super::og;

/// The most of a page an unfurl reads.
pub const MAX_BYTES: usize = 1 << 20;

fn https(base: &Url, raw: Option<String>) -> Option<String> {
    let rel = raw?;
    let url = base.join(rel.trim()).ok()?;
    (url.scheme() == "https").then(|| url.to_string())
}

async fn body(response: reqwest::Response) -> Option<String> {
    let html = response
        .headers()
        .get(http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.to_ascii_lowercase().starts_with("text/html"));
    if !html || !response.status().is_success() {
        return None;
    }
    let mut response = response;
    let mut bytes = Vec::new();
    while let Ok(Some(chunk)) = response.chunk().await {
        let room = MAX_BYTES.saturating_sub(bytes.len());
        bytes.extend_from_slice(&chunk[..chunk.len().min(room)]);
        if bytes.len() >= MAX_BYTES {
            break;
        }
    }
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// The preview `html`, read from `url`, describes.
pub fn describe(url: &Url, html: &str) -> Preview {
    let meta = og::parse(html);
    let fallback = Preview::host(url);
    let favicon = https(url, meta.favicon).or_else(|| https(url, Some("/favicon.ico".to_owned())));
    Preview {
        title: meta.title.unwrap_or(fallback.title),
        description: meta.description,
        image: https(url, meta.image),
        site: meta.site,
        favicon,
    }
}

/// The preview of `url`, fetched through the guarded client; the bare host when the URL may
/// not be fetched, the fetch fails or the page is not HTML.
pub async fn unfurl(client: &reqwest::Client, url: &Url) -> Preview {
    if !allowed(url) {
        return Preview::host(url);
    }
    let Ok(response) = client.get(url.as_str()).send().await else { return Preview::host(url) };
    match body(response).await {
        Some(html) => describe(url, &html),
        None => Preview::host(url),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previews_keep_https_images_and_resolve_relative_ones() {
        let url = Url::parse("https://example.com/blog/post").unwrap();
        let html = r#"<title>Post</title><meta property="og:image" content="/cover.png">
            <meta name="twitter:description" content="About it"><link rel="icon" href="http://x.test/i.ico">"#;
        let preview = describe(&url, html);
        assert_eq!(preview.title, "Post");
        assert_eq!(preview.description.as_deref(), Some("About it"));
        assert_eq!(preview.image.as_deref(), Some("https://example.com/cover.png"));
        assert_eq!(preview.favicon.as_deref(), Some("https://example.com/favicon.ico"));
        let bare = describe(&url, "<p>nothing</p>");
        assert_eq!(
            (bare.title.as_str(), bare.favicon.as_deref()),
            ("example.com", Some("https://example.com/favicon.ico"))
        );
    }
}
