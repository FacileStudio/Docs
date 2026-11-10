//! `api import-outline`: copies an Outline instance into one space, collections as top pages.

pub mod client;
pub mod convert;
pub mod plan;
mod write;

use std::path::Path;

use sqlx::PgPool;

use crate::modules::documents::markdown::{MARKDOWN_MAX_BYTES, within};

/// The parsed command line.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Args {
    pub url: String,
    pub token: String,
    pub space: String,
    pub dry_run: bool,
    /// The largest page markdown the import parses: `MARKDOWN_MAX_BYTES`.
    pub markdown_max: usize,
}

/// Usage, printed on a malformed command line.
pub const USAGE: &str = "usage: api import-outline --url https://docs.facile.studio --token <api token> \
    --space <registre space id> [--dry-run]   (the token may come from OUTLINE_TOKEN instead)";

/// Parses `import-outline` arguments; `None` when argv is another command.
pub fn parse(args: &[String]) -> Option<Result<Args, String>> {
    let (first, rest) = args.split_first()?;
    if first != "import-outline" {
        return None;
    }
    let token = std::env::var("OUTLINE_TOKEN").unwrap_or_default();
    let mut parsed = Args { token, markdown_max: MARKDOWN_MAX_BYTES, ..Args::default() };
    let mut iter = rest.iter();
    while let Some(flag) = iter.next() {
        let slot = match flag.as_str() {
            "--dry-run" => {
                parsed.dry_run = true;
                continue;
            }
            "--url" => &mut parsed.url,
            "--token" => &mut parsed.token,
            "--space" => &mut parsed.space,
            other => return Some(Err(format!("unknown flag {other}\n{USAGE}"))),
        };
        *slot = iter.next().cloned().unwrap_or_default();
    }
    let missing = parsed.url.is_empty() || parsed.token.is_empty() || parsed.space.is_empty();
    Some(if missing { Err(USAGE.to_owned()) } else { Ok(parsed) })
}

/// The dry-run report: the tree that would be written and every construct left as it was.
pub fn report(plan: &plan::Plan) -> String {
    let mut out = format!("{} pages\n", plan.pages.len());
    for page in &plan.pages {
        let depth =
            std::iter::successors(page.parent, |p| plan.pages.iter().find(|q| q.id == *p).and_then(|q| q.parent))
                .count();
        out.push_str(&format!("{}- {} ({} attachments)\n", "  ".repeat(depth), page.title, page.attachments.len()));
    }
    out.push_str(&format!("{} unconverted constructs\n", plan.unconverted.len()));
    for (title, what) in &plan.unconverted {
        out.push_str(&format!("  {title}: {what}\n"));
    }
    out
}

/// Refuses a plan holding a page over `max` bytes of markdown, naming every such page.
pub fn check_sizes(plan: &plan::Plan, max: usize) -> Result<(), String> {
    let over: Vec<String> = plan
        .pages
        .iter()
        .filter_map(|page| within(page.markdown.len(), max).err().map(|e| format!("{}: {}", page.title, e.message)))
        .collect();
    match over.is_empty() {
        true => Ok(()),
        false => Err(format!("pages over MARKDOWN_MAX_BYTES, split them in Outline first:\n  {}", over.join("\n  "))),
    }
}

/// Runs the import: prints the report, then writes unless `--dry-run`.
pub async fn run(pool: &PgPool, data: &Path, args: &Args) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = client::Client { base: args.url.clone(), token: args.token.clone(), http: reqwest::Client::new() };
    let plan = plan::build(&client).await?;
    check_sizes(&plan, args.markdown_max)?;
    let mut out = report(&plan);
    if !args.dry_run {
        let count = write::write(pool, &client, data, args, plan).await?;
        out.push_str(&format!("imported {count} pages into {}\n", args.space));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_command_line_needs_url_token_and_space() {
        assert_eq!(parse(&argv(&["migrate", "up"])), None);
        let ok = parse(&argv(&["import-outline", "--url", "u", "--token", "t", "--space", "s", "--dry-run"]));
        let want = Args { url: "u".into(), token: "t".into(), space: "s".into(), dry_run: true, markdown_max: 1 << 20 };
        assert_eq!(ok, Some(Ok(want)));
        assert!(parse(&argv(&["import-outline", "--url", "u"])).unwrap().is_err());
    }
}
