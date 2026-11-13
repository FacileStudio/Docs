use std::path::Path;

use tronc::http::{SecurityConfig, Sources};

/// The origins a page may frame: the suite apps whose embeds and the Bulle sidebar Docs shows.
pub const FRAMED: [&str; 6] = [
    "'self'",
    "https://toile.facile.studio",
    "https://feuilles.facile.studio",
    "https://vision.facile.studio",
    "https://bulle.facile.studio",
    "https://docs.facile.studio",
];

/// The CSP sources Docs' client needs on top of tronc's policy: the inline scripts
/// adapter-static writes, hashed from the build the binary serves, `'wasm-unsafe-eval'` for
/// encre's markdown WebAssembly, and the suite apps it frames. tronc's `img-src` already takes
/// any `https:` image, so unfurled previews and favicons need nothing more.
pub fn sources(client_dir: &Path) -> Sources {
    let mut scripts = vec!["'wasm-unsafe-eval'".to_owned()];
    if tronc::spa::available(client_dir) {
        match tronc::spa::inline_script_hashes(client_dir) {
            Ok(hashes) => scripts.extend(hashes),
            Err(error) => tracing::error!(%error, "failed to hash the client's inline scripts"),
        }
    }
    let frames = FRAMED.iter().map(|origin| (*origin).to_owned()).collect();
    Sources::from([("script-src".to_owned(), scripts), ("frame-src".to_owned(), frames)])
}

/// tronc's security configuration with [`sources`].
pub fn config(base: SecurityConfig, client_dir: &Path) -> SecurityConfig {
    SecurityConfig { csp_sources: sources(client_dir), ..base }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_policy_frames_the_suite_apps_and_takes_https_images() {
        let policy = tronc::http::build_csp(&super::sources(std::path::Path::new("/nonexistent")), "").unwrap();
        let directive = |name: &str| policy.split(';').map(str::trim).find(|d| d.starts_with(name)).unwrap().to_owned();
        assert_eq!(
            directive("frame-src "),
            "frame-src 'self' https://toile.facile.studio https://feuilles.facile.studio https://vision.facile.studio \
             https://bulle.facile.studio https://docs.facile.studio"
        );
        assert!(directive("img-src ").split(' ').any(|source| source == "https:"), "{policy}");
        assert!(directive("frame-ancestors ").ends_with("'none'"), "{policy}");
    }
}
