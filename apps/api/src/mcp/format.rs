/// The Facile Markdown and operations short form, from encre `docs/agents.md` at v0.5.0, as
/// every read and write tool description carries it.
pub const SHORT_FORM: &str = r#"Documents are Facile Markdown: CommonMark with GFM tables, task lists (- [ ] / - [x]) and
strikethrough (~~a~~). Raw HTML is plain text, except <br> as a line break. Headings go to
level 3 (#, ##, ###). No footnotes, math or front matter.

Custom blocks are backtick fences whose info string starts with the block name:

````callout tone="warning" title="Before you deploy"
Markdown inside, any blocks.
````

```stat
{"label": "Revenue", "value": "12 400 €", "delta": "+4%"}
```

```chart
{"kind": "bar", "title": "Signups", "labels": ["Mon", "Tue"], "series": [{"name": "web", "data": [3, 5]}]}
```

- callout: tone is neutral, info, success, warning or danger; title is optional. Its fence is
  one backtick longer than the longest backtick run inside it.
- stat and chart hold one JSON object. chart kind is bar or line; every series has one number
  per label.
- Attribute values use double quotes; \" and \\ are the only escapes. A malformed custom block
  stays a code block.

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

/// A tool description: what the tool does, then the short form.
pub fn described(what: &str) -> String {
    format!("{what}\n\n{SHORT_FORM}")
}
