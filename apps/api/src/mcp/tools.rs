use tronc::mcp::{Annotations, Confirmations, Tool, ToolSpec, tool};

use super::format::described;
use super::{read, replace, write};
use crate::state::AppState;

fn spec(name: &str, title: &str, what: &str, annotations: Annotations) -> ToolSpec {
    ToolSpec { name: name.into(), title: title.into(), description: described(what), annotations: Some(annotations) }
}

fn reading() -> Annotations {
    Annotations { read_only: true, destructive: false, idempotent: true, open_world: false }
}

fn writing(destructive: bool) -> Annotations {
    Annotations { read_only: false, destructive, idempotent: false, open_world: false }
}

fn reads(state: &AppState) -> Vec<Tool> {
    let (a, b, c, d) = (state.clone(), state.clone(), state.clone(), state.clone());
    vec![
        tool(
            spec(
                "docs_search",
                "Search pages",
                "Full-text search over the pages of one space (or your personal pages).",
                reading(),
            ),
            move |call, input| read::search(a.clone(), call, input),
        ),
        tool(
            spec(
                "docs_list_tree",
                "List the page tree",
                "Every page of one space, parents first, siblings in order.",
                reading(),
            ),
            move |call, input| read::list_tree(b.clone(), call, input),
        ),
        tool(
            spec("docs_read", "Read a page", "One page as Facile Markdown, with its version.", reading()),
            move |call, input| read::read(c.clone(), call, input),
        ),
        tool(
            spec("docs_read_blocks", "Read a page's blocks", "Top-level blocks with ids, for docs_edit.", reading()),
            move |call, input| read::read_blocks(d.clone(), call, input),
        ),
    ]
}

/// The seven tools.
pub fn all(state: &AppState, confirmations: &Confirmations) -> Vec<Tool> {
    let (a, b, c, confirm) = (state.clone(), state.clone(), state.clone(), confirmations.clone());
    let mut tools = reads(state);
    tools.push(tool(
        spec("docs_create", "Create a page", "A new page from Facile Markdown.", writing(false)),
        move |call, input| write::create(a.clone(), call, input),
    ));
    tools.push(tool(
        spec(
            "docs_edit",
            "Edit a page",
            "Applies operations (not replace_all) in order, all or nothing.",
            writing(false),
        ),
        move |call, input| write::edit(b.clone(), call, input),
    ));
    tools.push(tool(
        spec(
            "docs_replace",
            "Replace a whole page",
            "Overwrites the whole page. The person confirms first; nothing changes until they do.",
            writing(true),
        ),
        move |call, input| replace::replace(c.clone(), confirm.clone(), call, input),
    ));
    tools
}
