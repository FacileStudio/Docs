use std::collections::HashMap;

use uuid::Uuid;

/// What one document's markdown needs besides the dialect rewrite.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Converted {
    /// Facile Markdown.
    pub markdown: String,
    /// Outline attachment paths found, in order.
    pub attachments: Vec<String>,
    /// Constructs left as they were, for the dry-run report.
    pub unconverted: Vec<String>,
}

fn tone(kind: &str) -> Option<&'static str> {
    match kind {
        "info" => Some("info"),
        "warning" => Some("warning"),
        "tip" | "success" => Some("success"),
        _ => None,
    }
}

fn fence(body: &[String]) -> String {
    let longest = body.iter().flat_map(|line| line.split(|c| c != '`')).map(str::len).max().unwrap_or(0);
    "`".repeat((longest + 1).max(4))
}

/// Outline's `:::info` … `:::` notices as `callout` blocks. An unknown notice kind stays as it
/// was and is reported.
pub fn notices(markdown: &str, unconverted: &mut Vec<String>) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut open: Option<(&'static str, Vec<String>)> = None;
    for line in markdown.lines() {
        let trimmed = line.trim();
        match (&mut open, trimmed.strip_prefix(":::")) {
            (None, Some(kind)) if !kind.is_empty() => match tone(kind.trim()) {
                Some(tone) => open = Some((tone, Vec::new())),
                None => {
                    unconverted.push(format!("notice `:::{}`", kind.trim()));
                    out.push(line.to_owned());
                }
            },
            (Some(_), Some("")) => {
                let (tone, body) = open.take().unwrap_or_default();
                let fence = fence(&body);
                out.push(format!("{fence}callout tone=\"{tone}\""));
                out.extend(body);
                out.push(fence);
            }
            (Some((_, body)), _) => body.push(line.to_owned()),
            (None, _) => out.push(line.to_owned()),
        }
    }
    if let Some((_, body)) = open {
        unconverted.push("notice without a closing `:::`".to_owned());
        out.push(":::".to_owned());
        out.extend(body);
    }
    out.join("\n")
}

/// Rewrites every `/doc/<slug>-<urlId>` link to `/d/<new id>`; a link to a page that is not
/// imported is reported.
pub fn links(markdown: &str, ids: &HashMap<String, Uuid>, unconverted: &mut Vec<String>) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while let Some(at) = rest.find("](/doc/") {
        out.push_str(&rest[..at + 2]);
        let tail = &rest[at + 2..];
        let end = tail.find([')', ' ', '#']).unwrap_or(tail.len());
        let target = &tail[..end];
        let url_id = target.rsplit('-').next().unwrap_or_default();
        match ids.get(url_id) {
            Some(id) => out.push_str(&format!("/d/{id}")),
            None => {
                unconverted.push(format!("link to a page outside the import: {target}"));
                out.push_str(target);
            }
        }
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

/// The `/api/attachments.redirect?id=…` paths in `markdown`.
pub fn attachment_paths(markdown: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = markdown;
    while let Some(at) = rest.find("/api/attachments.redirect?id=") {
        let tail = &rest[at..];
        let end = tail.find([')', ' ', '"', '\n']).unwrap_or(tail.len());
        found.push(tail[..end].to_owned());
        rest = &tail[end..];
    }
    found
}

/// One document's markdown in Facile Markdown, with what is left to do.
pub fn convert(markdown: &str, ids: &HashMap<String, Uuid>) -> Converted {
    let mut unconverted = Vec::new();
    let markdown = links(&notices(markdown, &mut unconverted), ids, &mut unconverted);
    let attachments = attachment_paths(&markdown);
    Converted { markdown, attachments, unconverted }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notices_become_callouts_with_a_long_enough_fence() {
        let mut left = Vec::new();
        let md = notices(":::info\nRun ```this```\n:::\n\n:::odd\nx\n:::", &mut left);
        assert_eq!(md, "````callout tone=\"info\"\nRun ```this```\n````\n\n:::odd\nx\n:::");
        assert_eq!(left, ["notice `:::odd`"]);
    }

    #[test]
    fn internal_links_and_attachments_are_found() {
        let new = Uuid::new_v4();
        let ids = HashMap::from([("aBc123XyZ0".to_owned(), new)]);
        let done =
            convert("[a](/doc/plan-aBc123XyZ0) [b](/doc/gone-zzzzzzzzzz) ![i](/api/attachments.redirect?id=42)", &ids);
        assert_eq!(
            done.markdown,
            format!("[a](/d/{new}) [b](/doc/gone-zzzzzzzzzz) ![i](/api/attachments.redirect?id=42)")
        );
        assert_eq!(done.attachments, ["/api/attachments.redirect?id=42"]);
        assert_eq!(done.unconverted, ["link to a page outside the import: /doc/gone-zzzzzzzzzz"]);
    }
}
