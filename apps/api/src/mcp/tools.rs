use tronc::mcp::{Annotations, Confirmations, Tool, ToolSpec, tool};

use super::format::described;
use super::{manage, read, replace, write};
use crate::state::AppState;

const SEARCH: &str = "Full-text search over the pages of one space (or your personal pages). Call it to find a \
page by its words when you do not know its id.";
const LIST_TREE: &str = "The live pages of one space (or your personal pages), parents first, siblings in order, \
50 at a time by default. Call it to browse a space or to find a parent for docs_create or docs_move; pass \
next_cursor as cursor for the next window.";
const LIST_TRASH: &str = "The trashed pages of one space (or your personal pages), 50 at a time by default. Call \
it to find a page to bring back with docs_restore. The trash empties itself 30 days after a page went in.";
const READ: &str = "One page as Facile Markdown, with its version. Call it before you rewrite a page.";
const READ_BLOCKS: &str = "Top-level blocks with their ids. Call it before docs_edit, which addresses blocks by id. \
A long page comes back in parts: when truncated is true, pass next_range as range for the rest.";
const CREATE: &str = "A new page from Facile Markdown. Call it once per page: each call adds another page.";
const EDIT: &str = "Applies operations (not replace_all) in order, all or nothing. Call it for any change inside a \
page; replace_block and delete_block remove what they replace.";
const REPLACE: &str = "Overwrites the whole page. Call it only when most of the page changes; the person confirms \
first and nothing changes until they do.";
const UPDATE: &str = "Changes a page's title, its icon or both; the content is not touched. Call it to rename a \
page. The old title is replaced.";
const MOVE: &str = "Moves a page and its subtree under another parent of the same space, or to the top, before a \
given sibling or last. Call it to reorganize the tree; a page cannot move into another space or inside itself.";
const TRASH: &str = "Moves a page and its subtree to the trash and closes it for everyone editing it. Call it to \
delete a page; docs_restore brings it back within 30 days, after which it is gone for good.";
const RESTORE: &str = "Brings a trashed page back with what was trashed with it; it goes to the top when its \
parent is still in the trash. Call it to undo docs_trash. A page that is not in the trash is answered unchanged.";

fn spec(name: &str, title: &str, description: impl Into<String>, annotations: Annotations) -> ToolSpec {
    ToolSpec { name: name.into(), title: title.into(), description: description.into(), annotations: Some(annotations) }
}

fn browse(state: &AppState) -> Vec<Tool> {
    let (a, b, c) = (state.clone(), state.clone(), state.clone());
    vec![
        tool(spec("docs_search", "Search pages", SEARCH, Annotations::READ), move |call, input| {
            read::search(a.clone(), call, input)
        }),
        tool(spec("docs_list_tree", "List the page tree", LIST_TREE, Annotations::READ), move |call, input| {
            read::list_tree(b.clone(), call, input)
        }),
        tool(spec("docs_list_trash", "List the trash", LIST_TRASH, Annotations::READ), move |call, input| {
            manage::list_trash(c.clone(), call, input)
        }),
    ]
}

fn reads(state: &AppState) -> Vec<Tool> {
    let (a, b) = (state.clone(), state.clone());
    vec![
        tool(spec("docs_read", "Read a page", described(READ), Annotations::READ), move |call, input| {
            read::read(a.clone(), call, input)
        }),
        tool(
            spec("docs_read_blocks", "Read a page's blocks", described(READ_BLOCKS), Annotations::READ),
            move |call, input| read::read_blocks(b.clone(), call, input),
        ),
    ]
}

fn content(state: &AppState, confirmations: &Confirmations) -> Vec<Tool> {
    let (a, b, c, confirm) = (state.clone(), state.clone(), state.clone(), confirmations.clone());
    vec![
        tool(spec("docs_create", "Create a page", described(CREATE), Annotations::ADD), move |call, input| {
            write::create(a.clone(), call, input)
        }),
        tool(spec("docs_edit", "Edit a page", described(EDIT), Annotations::REMOVE), move |call, input| {
            write::edit(b.clone(), call, input)
        }),
        tool(
            spec("docs_replace", "Replace a whole page", described(REPLACE), Annotations::REMOVE),
            move |call, input| replace::replace(c.clone(), confirm.clone(), call, input),
        ),
    ]
}

fn organize(state: &AppState) -> Vec<Tool> {
    let (a, b, c, d) = (state.clone(), state.clone(), state.clone(), state.clone());
    vec![
        tool(spec("docs_update", "Rename a page", UPDATE, Annotations::OVERWRITE), move |call, input| {
            manage::update(a.clone(), call, input)
        }),
        tool(spec("docs_move", "Move a page", MOVE, Annotations::OVERWRITE), move |call, input| {
            manage::move_to(b.clone(), call, input)
        }),
        tool(spec("docs_trash", "Trash a page", TRASH, Annotations::OVERWRITE), move |call, input| {
            manage::trash(c.clone(), call, input)
        }),
        tool(spec("docs_restore", "Restore a page", RESTORE, Annotations::SET), move |call, input| {
            manage::restore(d.clone(), call, input)
        }),
    ]
}

/// The twelve tools.
pub fn all(state: &AppState, confirmations: &Confirmations) -> Vec<Tool> {
    let mut tools = browse(state);
    tools.extend(reads(state));
    tools.extend(content(state, confirmations));
    tools.extend(organize(state));
    tools
}
