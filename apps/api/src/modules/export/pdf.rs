use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use encre::Node;
use encre::pdf::Assets;
use encre::typst::RenderOptions;
use tokio::sync::Semaphore;
use tronc::{Error, Result};
use uuid::Uuid;

use crate::modules::attachments::service as attachments;
use crate::modules::documents::access::{self, Access};
use crate::state::AppState;

/// How long one compile may take.
pub const TIMEOUT: Duration = Duration::from_secs(30);
/// How many compiles run at once.
pub const CONCURRENCY: usize = 2;
/// The most distinct images one export embeds.
pub const MAX_IMAGES: usize = 64;
/// The most image bytes one export loads.
pub const MAX_IMAGE_BYTES: usize = 100 << 20;

/// The compile slots, shared by the whole process.
pub fn slots() -> Arc<Semaphore> {
    Arc::new(Semaphore::new(CONCURRENCY))
}

fn image_sources(node: &Node, out: &mut Vec<String>) {
    let src = node.attr_str("src");
    if node.kind == "image" && !out.iter().any(|seen| seen == src) {
        out.push(src.to_owned());
    }
    node.content.iter().for_each(|child| image_sources(child, out));
}

/// The attachment id an image `src` names, when it is one of this app's.
pub fn attachment_id(src: &str) -> Option<Uuid> {
    let rest = src.strip_prefix("/api/attachments/")?;
    Uuid::parse_str(rest.split('?').next()?).ok()
}

async fn images(state: &AppState, user_id: i64, doc: &Node) -> Result<(HashMap<String, PathBuf>, Assets)> {
    let mut sources = Vec::new();
    image_sources(doc, &mut sources);
    if sources.len() > MAX_IMAGES {
        return Err(Error::too_large(format!("a PDF embeds at most {MAX_IMAGES} images")));
    }
    let (mut paths, mut assets, mut total) = (HashMap::new(), Assets::new(), 0usize);
    for src in sources {
        let Some(id) = attachment_id(&src) else { continue };
        let Ok(found) = attachments::find(state, id).await else { continue };
        if !found.mime.starts_with("image/")
            || access::access(state, user_id, found.document_id).await? < Access::Viewer
        {
            continue;
        }
        let Ok((_, bytes)) = attachments::open(state, Some(user_id), None, id).await else { continue };
        total += bytes.len();
        if total > MAX_IMAGE_BYTES {
            return Err(Error::too_large("the images of this PDF exceed 100 MiB"));
        }
        let path = format!("/images/{id}.{}", found.mime.trim_start_matches("image/"));
        assets.insert(&path, bytes);
        paths.insert(src, PathBuf::from(path));
    }
    Ok((paths, assets))
}

fn compile(doc: &Node, title: &str, paths: &HashMap<String, PathBuf>, assets: &Assets) -> Result<Vec<u8>> {
    let resolve = |src: &str| paths.get(src).cloned();
    let options = RenderOptions {
        title: title.to_owned(),
        footer: Some(title.to_owned()),
        image: Some(&resolve),
        ..RenderOptions::default()
    };
    let source = encre::typst::render(doc, &options);
    encre::pdf::compile(&source, assets).map_err(|error| Error::internal("the PDF did not compile", error))
}

/// The document as a PDF: a compile slot first, then images resolved through the caller's
/// access (at most [`MAX_IMAGES`] and [`MAX_IMAGE_BYTES`]), compiled off the async threads, at
/// most [`CONCURRENCY`] at once, refused after [`TIMEOUT`].
pub async fn export(state: &AppState, user_id: i64, doc: Node, title: String) -> Result<Vec<u8>> {
    let permit = state.exports.clone().acquire_owned().await.map_err(|e| Error::internal("export slots closed", e))?;
    let (paths, assets) = images(state, user_id, &doc).await?;
    let job = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        compile(&doc, &title, &paths, &assets)
    });
    match tokio::time::timeout(TIMEOUT, job).await {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => Err(Error::internal("the PDF compile panicked", error)),
        Err(_) => Err(Error::unavailable("the PDF took longer than 30 seconds")),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_local_attachment_urls_name_an_attachment() {
        let id = uuid::Uuid::new_v4();
        assert_eq!(super::attachment_id(&format!("/api/attachments/{id}")), Some(id));
        assert_eq!(super::attachment_id(&format!("/api/attachments/{id}?share=t")), Some(id));
        assert_eq!(super::attachment_id("https://example.test/a.png"), None);
    }
}
