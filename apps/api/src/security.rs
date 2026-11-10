use std::path::Path;

use tronc::http::{SecurityConfig, Sources};

/// The CSP sources Docs' client needs on top of tronc's policy: the inline scripts
/// adapter-static writes, hashed from the build the binary serves, and `'wasm-unsafe-eval'`
/// for encre's markdown WebAssembly.
pub fn sources(client_dir: &Path) -> Sources {
    let mut scripts = vec!["'wasm-unsafe-eval'".to_owned()];
    if tronc::spa::available(client_dir) {
        match tronc::spa::inline_script_hashes(client_dir) {
            Ok(hashes) => scripts.extend(hashes),
            Err(error) => tracing::error!(%error, "failed to hash the client's inline scripts"),
        }
    }
    Sources::from([("script-src".to_owned(), scripts)])
}

/// tronc's security configuration with [`sources`].
pub fn config(base: SecurityConfig, client_dir: &Path) -> SecurityConfig {
    SecurityConfig { csp_sources: sources(client_dir), ..base }
}
