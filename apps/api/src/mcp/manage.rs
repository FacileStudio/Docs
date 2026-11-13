use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, ToolError, failure};

use super::page::{Listed, Window, window};
use super::user;
use crate::extract::id;
use crate::modules::documents::access::{self, Access};
use crate::modules::documents::types::{Document, MoveRequest, PatchRequest, ScopeQuery};
use crate::modules::documents::{service, trash, tree};
use crate::state::AppState;

const HINT: &str = "docs_list_tree lists live page ids, docs_list_trash trashed ones.";

fn found(error: tronc::Error) -> ToolError {
    failure(error, HINT)
}

/// One page after a write: where it sits now.
#[derive(Serialize, JsonSchema)]
pub struct Placed {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub icon: String,
    pub version: i64,
    pub trashed: bool,
}

impl From<Document> for Placed {
    fn from(doc: Document) -> Self {
        Placed {
            id: doc.id.to_string(),
            parent_id: doc.parent_id.map(|p| p.to_string()),
            title: doc.title,
            icon: doc.icon,
            version: doc.version,
            trashed: doc.stamps.trashed_at.is_some(),
        }
    }
}

/// `docs_update`.
#[derive(Deserialize, JsonSchema)]
pub struct UpdateIn {
    /// The page id.
    pub document_id: String,
    /// The new title, at most 200 characters; unchanged when absent.
    pub title: Option<String>,
    /// The new icon, an emoji or an icon name of at most 32 characters; unchanged when absent.
    pub icon: Option<String>,
}

/// `docs_update`: renames a page or changes its icon. The same call twice leaves the same page.
pub async fn update(state: AppState, call: Call, input: UpdateIn) -> Result<Answer<Placed>, ToolError> {
    if input.title.is_none() && input.icon.is_none() {
        return Err(ToolError("give title, icon or both: nothing to change".into()));
    }
    let request = PatchRequest { title: input.title, icon: input.icon };
    let doc =
        service::patch(&state, user(&call)?, id(&input.document_id).map_err(found)?, request).await.map_err(found)?;
    Ok(Placed::from(doc).into())
}

/// `docs_move`.
#[derive(Deserialize, JsonSchema)]
pub struct MoveIn {
    /// The page id.
    pub document_id: String,
    /// The new parent page, in the same space; the top of the tree when absent.
    pub parent_id: Option<String>,
    /// The sibling the page goes right before; the last place among its siblings when absent.
    pub before_id: Option<String>,
}

/// `docs_move`: a new parent and place in the same space, with its subtree.
pub async fn move_to(state: AppState, call: Call, input: MoveIn) -> Result<Answer<Placed>, ToolError> {
    let parse = |raw: Option<&str>, field: &str| {
        raw.map(|raw| uuid::Uuid::parse_str(raw).map_err(|_| ToolError(format!("{field} is not a page id: {raw}"))))
            .transpose()
    };
    let request = MoveRequest {
        parent_id: parse(input.parent_id.as_deref(), "parent_id")?,
        before: parse(input.before_id.as_deref(), "before_id")?,
    };
    let doc =
        tree::move_to(&state, user(&call)?, id(&input.document_id).map_err(found)?, request).await.map_err(found)?;
    Ok(Placed::from(doc).into())
}

/// `docs_trash` and `docs_restore`.
#[derive(Deserialize, JsonSchema)]
pub struct PageIn {
    /// The page id.
    pub document_id: String,
}

/// `docs_trash`: the page and its subtree go to the trash; `docs_restore` brings them back.
pub async fn trash(state: AppState, call: Call, input: PageIn) -> Result<Answer<Placed>, ToolError> {
    let (user, document) = (user(&call)?, id(&input.document_id).map_err(found)?);
    trash::trash(&state, user, document).await.map_err(found)?;
    let have = access::check(&state, user, document, Access::Viewer).await.map_err(found)?;
    Ok(Placed::from(service::answer(service::fetch(&state, document).await?, have, false)?).into())
}

/// `docs_restore`: brings back a trashed page and what was trashed with it. A page that is not
/// in the trash is answered as it is.
pub async fn restore(state: AppState, call: Call, input: PageIn) -> Result<Answer<Placed>, ToolError> {
    let (user, document) = (user(&call)?, id(&input.document_id).map_err(found)?);
    let have = access::check(&state, user, document, Access::Manager).await.map_err(found)?;
    let row = service::fetch(&state, document).await?;
    let doc = match row.trashed_at {
        Some(_) => trash::restore(&state, user, document).await.map_err(found)?,
        None => service::answer(row, have, false)?,
    };
    Ok(Placed::from(doc).into())
}

/// `docs_list_trash`.
#[derive(Deserialize, JsonSchema)]
pub struct TrashIn {
    /// The space; personal pages when absent.
    pub space_id: Option<String>,
    /// The `next_cursor` of the previous call; the first pages when absent.
    pub cursor: Option<String>,
    /// How many pages, 1 to 200; 50 when absent.
    pub limit: Option<usize>,
}

/// One trashed page.
#[derive(Serialize, JsonSchema)]
pub struct Trashed {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
    /// RFC 3339; the trash empties itself 30 days later.
    pub trashed_at: String,
}

/// `docs_list_trash`: the trashed pages of one scope, a window at a time.
pub async fn list_trash(state: AppState, call: Call, input: TrashIn) -> Result<Answer<Listed<Trashed>>, ToolError> {
    let query = ScopeQuery { space_id: input.space_id, trashed: true };
    let entries = tree::list(&state, user(&call)?, &query).await?;
    let pages = entries
        .into_iter()
        .map(|e| Trashed {
            id: e.id.to_string(),
            parent_id: e.parent_id.map(|p| p.to_string()),
            title: e.title,
            trashed_at: e
                .trashed_at
                .and_then(|at| at.format(&time::format_description::well_known::Rfc3339).ok())
                .unwrap_or_default(),
        })
        .collect();
    Ok(window(pages, Window { cursor: input.cursor, limit: input.limit })?.into())
}
