use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;
use time::OffsetDateTime;
use uuid::Uuid;

/// The columns every document answer reads, as a literal for `concat!`.
macro_rules! columns {
    () => {
        "id, space_id, parent_id, position, title, icon, schema_version, version, content, created_at, updated_at, \
         trashed_at"
    };
}
pub(crate) use columns;

/// One `documents` row, without the derived columns.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DocumentRow {
    pub id: Uuid,
    pub space_id: Option<String>,
    pub parent_id: Option<Uuid>,
    pub position: f64,
    pub title: String,
    pub icon: String,
    pub schema_version: i32,
    pub version: i64,
    pub content: Json<Value>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub trashed_at: Option<OffsetDateTime>,
}

/// A document as the API answers it: `content` (encre JSON) or `markdown`, never both.
#[derive(Debug, Serialize)]
pub struct Document {
    pub id: Uuid,
    pub space_id: Option<String>,
    pub parent_id: Option<Uuid>,
    pub position: f64,
    pub title: String,
    pub icon: String,
    pub schema_version: i32,
    pub version: i64,
    #[serde(flatten)]
    pub stamps: Stamps,
    pub access: &'static str,
    #[serde(flatten)]
    pub body: Body,
}

/// When a document was created, last changed and trashed.
#[derive(Debug, Serialize)]
pub struct Stamps {
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub trashed_at: Option<OffsetDateTime>,
}

/// The document body in the format asked for.
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Body {
    /// encre JSON.
    Content(Value),
    /// Facile Markdown.
    Markdown(String),
}

/// One entry of the tree: no content.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TreeEntry {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub position: f64,
    pub title: String,
    pub icon: String,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option", skip_serializing_if = "Option::is_none")]
    pub trashed_at: Option<OffsetDateTime>,
}

/// `POST /documents`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRequest {
    #[serde(default)]
    pub title: String,
    pub space_id: Option<String>,
    pub parent_id: Option<Uuid>,
    pub markdown: Option<String>,
    pub content: Option<Value>,
}

/// `POST /documents/import-openui`: an OpenUI program saved as a page.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenUiRequest {
    pub source: String,
    #[serde(default)]
    pub title: String,
    pub space_id: Option<String>,
    pub parent_id: Option<Uuid>,
}

/// `PATCH /documents/{id}`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchRequest {
    pub title: Option<String>,
    pub icon: Option<String>,
}

/// `POST /documents/{id}/move`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoveRequest {
    pub parent_id: Option<Uuid>,
    pub before: Option<Uuid>,
}

/// `POST /documents/{id}/ops`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpsRequest {
    pub ops: Vec<encre::ops::Op>,
}

/// What `POST /documents/{id}/ops` answers.
#[derive(Debug, Serialize)]
pub struct OpsAnswer {
    pub version: i64,
    pub markdown: String,
}

/// `?space_id=&trashed=` on the tree.
#[derive(Debug, Deserialize)]
pub struct ScopeQuery {
    pub space_id: Option<String>,
    #[serde(default)]
    pub trashed: bool,
}

/// `?format=json|markdown` on a read.
#[derive(Debug, Deserialize)]
pub struct FormatQuery {
    pub format: Option<String>,
}
