/// The Facile Markdown and operations short form, from encre `docs/agents.md` at v0.9.0, as
/// every read and write tool description carries it.
pub const SHORT_FORM: &str = r#"Documents are Facile Markdown: CommonMark with GFM tables, task lists (- [ ] / - [x]) and
strikethrough (~~a~~). Raw HTML is plain text, except <br> as a line break. Headings go to
level 3 (#, ##, ###). No footnotes, math or front matter.

Custom blocks are backtick fences whose info string starts with the block name:

````callout tone="warning" title="Before you deploy"
Markdown inside, any blocks.
````

```stat
{"label": "Revenue", "value": "12 400 €", "delta": "+4%", "sub": "This month"}
```

```chart
{"kind": "bar", "title": "Signups", "labels": ["Mon", "Tue"], "series": [{"name": "web", "data": [3, 5]}]}
```

```mermaid
flowchart LR
  A --> B
```

````columns
```column title="Pros" tone="success"
Fast to ship
```

```column title="Cons" tone="danger"
More to read
```
````

- callout: tone is neutral, info, success, warning or danger; title is optional. Its fence is
  one backtick longer than the longest backtick run inside it. A GFM alert (> [!NOTE], [!TIP],
  [!IMPORTANT], [!WARNING], [!CAUTION], [!PROS], [!CONS]) is read as a callout.
- mermaid, d2 and dot fences are diagrams the app draws. columns holds 2 to 4 column fences;
  tone is neutral, success, danger, warning or info. A compare fence with ### sections is read
  as columns.
- stat and chart hold one JSON object. chart kind is bar or line; every series has one number
  per label.
- Attribute values use double quotes; \" and \\ are the only escapes. A malformed custom block
  stays a code block.
- Inline: ==highlight==, [badge:success Shipped] (tones neutral, accent, info, success, warning,
  danger) and [status:done] (a dot badge; also active, ready, error, failed, pending, running).

Read with read (whole document) or read_blocks (top-level blocks with their ids). Write with
one operation per call:

- append {markdown} / prepend {markdown}: add blocks at the end or start
- insert_after {block_id, markdown}: add blocks after one block
- replace_block {block_id, markdown}: replace one block; a heading's section is not touched
- delete_block {block_id}: remove one block
- patch {find, replace, expect=1}: replace exact text of a block's markdown, as read_blocks
  shows it; fails unless exactly `expect` matches exist; a match never spans two blocks
- replace_all {markdown, expect_version}: replace everything, only if the version still matches

Operations address top-level blocks only: a nested id is refused with its parent's id. A
refused operation changes nothing, and its error says what matched and how to narrow the
request. Writes answer the ids of the blocks they inserted. Block ids never appear in markdown.
Prefer insert_after, replace_block and patch over replace_all: they keep other people's cursors."#;

/// The limits Docs and encre v0.9.0 enforce, for every tool description.
pub const LIMITS: &str = "Limits: one call parses at most 1 MiB of markdown by default; a larger body \
is refused, so split it over several append calls. Blocks and inline spans nest at most 64 levels in \
markdown and 48 in the document, and one path holds at most 8 callout, columns and column blocks; deeper \
markdown is refused and changes nothing. A page stays under 5 MiB.";

/// A tool description: what the tool does, then the short form and the limits.
pub fn described(what: &str) -> String {
    format!("{what}\n\n{SHORT_FORM}\n\n{LIMITS}")
}

#[cfg(test)]
mod tests {
    #[test]
    fn descriptions_teach_the_schema_2_syntax_and_the_limits() {
        let text = super::described("Edit a page.");
        for wanted in ["==highlight==", "[badge:success", "[status:done]", "[!NOTE]", "```mermaid", "````columns"] {
            assert!(text.contains(wanted), "{wanted} missing");
        }
        assert!(text.contains("1 MiB") && text.contains("at most 8 callout"));
    }
}
