use std::path::Path;

use tronc::http::{SecurityConfig, Sources};

/// The origins a page may frame besides the Bulle sidebar: the suite apps whose embeds Docs shows.
pub const FRAMED: [&str; 5] = [
    "'self'",
    "https://toile.facile.studio",
    "https://feuilles.facile.studio",
    "https://vision.facile.studio",
    "https://docs.facile.studio",
];

/// The CSP sources Docs' client needs on top of tronc's policy: the inline scripts
/// adapter-static writes, hashed from the build the binary serves, `'wasm-unsafe-eval'` for
/// encre's markdown WebAssembly, Vision's tracker script, the suite apps it frames and the Bulle
/// sidebar at `bulle`. tronc's `img-src` already takes any `https:` image, so unfurled previews and
/// favicons need nothing more.
pub fn sources(client_dir: &Path, bulle: &str) -> Sources {
    let mut scripts = vec!["'wasm-unsafe-eval'".to_owned(), "https://vision.facile.studio".to_owned()];
    if tronc::spa::available(client_dir) {
        match tronc::spa::inline_script_hashes(client_dir) {
            Ok(hashes) => scripts.extend(hashes),
            Err(error) => tracing::error!(%error, "failed to hash the client's inline scripts"),
        }
    }
    let mut frames: Vec<String> = FRAMED.iter().map(|origin| (*origin).to_owned()).collect();
    frames.push(bulle.to_owned());
    Sources::from([("script-src".to_owned(), scripts), ("frame-src".to_owned(), frames)])
}

/// tronc's security configuration with [`sources`].
pub fn config(base: SecurityConfig, client_dir: &Path, bulle: &str) -> SecurityConfig {
    SecurityConfig { csp_sources: sources(client_dir, bulle), ..base }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_policy_frames_the_suite_apps_and_takes_https_images() {
        let policy =
            tronc::http::build_csp(&super::sources(std::path::Path::new("/nonexistent"), "https://bulle.test"), "")
                .unwrap();
        let directive = |name: &str| policy.split(';').map(str::trim).find(|d| d.starts_with(name)).unwrap().to_owned();
        assert_eq!(
            directive("frame-src "),
            "frame-src 'self' https://toile.facile.studio https://feuilles.facile.studio https://vision.facile.studio \
             https://docs.facile.studio https://bulle.test"
        );
        assert!(directive("img-src ").split(' ').any(|source| source == "https:"), "{policy}");
        assert!(directive("frame-ancestors ").ends_with("'none'"), "{policy}");
    }

    #[tokio::test]
    async fn nothing_frames_docs_the_confirm_page_in_any_spelling_included() {
        use tower::ServiceExt;
        let dir = std::env::temp_dir().join(format!("docs-spa-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("index.html"), "<!doctype html><title>Docs</title>").unwrap();
        let http = tronc::http::Config {
            security: super::config(Default::default(), &dir, "https://bulle.test"),
            ..Default::default()
        };
        let app = axum::Router::new()
            .route("/api/mcp/pending", axum::routing::get(|| async { "{}" }))
            .route("/mcp", axum::routing::post(|| async { "{}" }))
            .fallback_service(tronc::spa::handler(tronc::spa::Config { dir: dir.clone(), ..Default::default() }))
            .layer(tronc::http::layers(http).unwrap());
        for (method, path) in [
            ("GET", "/pending"),
            ("GET", "/pending/"),
            ("GET", "/pend%69ng?action=x"),
            ("GET", "/"),
            ("GET", "/api/mcp/pending"),
            ("POST", "/mcp"),
        ] {
            let request = http::Request::builder().method(method).uri(path).body(axum::body::Body::empty()).unwrap();
            let answer = app.clone().oneshot(request).await.unwrap();
            let policy = answer.headers()["content-security-policy"].to_str().unwrap().to_owned();
            assert!(policy.contains("frame-ancestors 'none'"), "{path}: {policy}");
            assert_eq!(answer.headers()["x-frame-options"], "DENY", "{path}");
        }
        std::fs::remove_dir_all(dir).ok();
    }
}
