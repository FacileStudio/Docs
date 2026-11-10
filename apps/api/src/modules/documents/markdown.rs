//! Reading Facile Markdown from a request: the size cap, a blocking thread, and refusals that
//! name the line and column.

use encre::Node;
use tronc::{Error, Result};

use super::content::empty;

/// The default cap on one markdown body the API parses: 1 MiB, about 0.6 s of encre CPU at worst.
pub const MARKDOWN_MAX_BYTES: usize = 1 << 20;

/// Refuses `bytes` of markdown over `max` with `resource_exhausted`.
pub fn within(bytes: usize, max: usize) -> Result<()> {
    match bytes > max {
        true => Err(Error::too_large(format!("the markdown is {bytes} bytes, over the {max}-byte limit"))),
        false => Ok(()),
    }
}

/// [`from_markdown`] on a blocking thread, after refusing markdown over `max` bytes.
pub async fn parse(markdown: String, max: usize) -> Result<Node> {
    within(markdown.len(), max)?;
    tokio::task::spawn_blocking(move || from_markdown(&markdown))
        .await
        .map_err(|error| Error::internal("the markdown parse did not finish", error))?
}

/// Reads and validates Facile Markdown; blank markdown is an empty page. A refusal names the line
/// and column of the block at fault.
pub fn from_markdown(markdown: &str) -> Result<Node> {
    let node = encre::markdown::validate(markdown).map_err(|error| {
        Error::invalid(format!("markdown at line {}, column {}: {}", error.line, error.column, error.message))
    })?;
    Ok(if node.content.is_empty() { empty() } else { node })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_past_the_nesting_limit_names_its_line_and_column() {
        let deep = format!("intro\n\n{}x", "> ".repeat(70));
        let error = from_markdown(&deep).unwrap_err();
        assert_eq!(error.code, tronc::Code::InvalidArgument);
        assert!(error.message.starts_with("markdown at line 3, column "), "{}", error.message);
    }

    #[tokio::test]
    async fn markdown_over_the_cap_is_refused_before_it_is_read() {
        let error = parse("a".repeat(65), 64).await.unwrap_err();
        assert_eq!(error.code, tronc::Code::ResourceExhausted);
        assert!(parse("a".repeat(64), 64).await.is_ok());
    }
}
