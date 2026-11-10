//! The API reference at `/docs`: one row per route, checked against the router by a test.

use utoipa::openapi::path::{HttpMethod, OperationBuilder, PathItem};
use utoipa::openapi::response::ResponseBuilder;
use utoipa::openapi::{InfoBuilder, OpenApi, OpenApiBuilder, PathsBuilder};

/// One documented route: method, path under `/api` (or the root), tag, summary, success status.
type Row = (&'static str, &'static str, &'static str, &'static str, &'static str);

const ROUTES: &[Row] = &[
    ("GET", "/version", "meta", "The release this binary was built from", "200"),
    ("GET", "/auth/config", "auth", "Whether sign-in is SSO only and OIDC is on", "200"),
    ("GET", "/auth/oidc", "auth", "Starts Registre sign-in (302)", "302"),
    ("GET", "/auth/oidc/callback", "auth", "Registre's redirect back, sets the session cookie (302)", "302"),
    ("POST", "/auth/oidc/exchange", "auth", "Trades a CLI one-time code for a session token", "200"),
    ("POST", "/auth/oidc/device/exchange", "auth", "Trades a facile-cli device token for a session token", "200"),
    ("POST", "/auth/sync-profile", "auth", "Refreshes the profile from Registre", "200"),
    ("POST", "/auth/backchannel-logout", "auth", "Registre's back-channel logout", "200"),
    ("POST", "/auth/logout", "auth", "Ends the session", "200"),
    ("GET", "/auth/me", "auth", "The signed-in person, or 401", "200"),
    ("GET", "/auth/session", "auth", "The signed-in person, or {\"user\": null}", "200"),
    ("POST", "/registre/sync", "registre", "Registre's directory nudge", "202"),
    ("POST", "/registre/deletions", "registre", "Registre's signed account deletion", "202"),
    ("GET", "/spaces", "spaces", "The caller's spaces and roles", "200"),
    ("GET", "/documents", "documents", "The page tree of a scope (?space_id=&trashed=)", "200"),
    ("POST", "/documents", "documents", "Creates a page from markdown or encre JSON", "201"),
    ("GET", "/documents/{id}", "documents", "One page (?format=json|markdown)", "200"),
    ("PATCH", "/documents/{id}", "documents", "Renames a page or sets its icon", "200"),
    ("DELETE", "/documents/{id}", "documents", "Moves a page and its subtree to the trash", "204"),
    ("POST", "/documents/{id}/ops", "documents", "Applies agent operations, live in open editors", "200"),
    ("POST", "/documents/{id}/move", "documents", "Moves a page under a parent, before a sibling", "200"),
    ("POST", "/documents/{id}/restore", "documents", "Restores a trashed page", "200"),
    ("DELETE", "/documents/{id}/permanent", "documents", "Deletes a trashed page for good", "204"),
    ("GET", "/documents/{id}/revisions", "documents", "Revisions, newest first", "200"),
    ("POST", "/documents/{id}/revisions/{rev}/restore", "documents", "Restores a revision's content", "200"),
    ("POST", "/documents/import-openui", "documents", "Creates a page from an OpenUI program", "201"),
    ("GET", "/documents/{id}/ws", "documents", "The y-sync WebSocket of one page", "101"),
    ("GET", "/documents/{id}/export.md", "export", "The page as Facile Markdown", "200"),
    ("GET", "/documents/{id}/export.commonmark", "export", "The page as portable CommonMark", "200"),
    ("GET", "/documents/{id}/export.pdf", "export", "The page as a PDF", "200"),
    ("POST", "/documents/{id}/attachments", "attachments", "Uploads a file (multipart `file`)", "201"),
    ("GET", "/attachments/{id}", "attachments", "Serves a file, sandboxed (?share= for public pages)", "200"),
    ("GET", "/search", "search", "Full-text search in one scope (?q=&space_id=)", "200"),
    ("POST", "/documents/{id}/share", "share", "Creates a share link; the token is shown once", "201"),
    ("GET", "/documents/{id}/share", "share", "The live share links of a page", "200"),
    ("DELETE", "/share/{id}", "share", "Revokes a share link", "204"),
    ("GET", "/public/{token}", "public", "A shared page, no session, rate limited", "200"),
    ("GET", "/public/{token}/documents/{id}", "public", "A page of a shared subtree", "200"),
    ("GET", "/mcp/actions", "mcp", "The caller's pending agent actions", "200"),
    ("POST", "/mcp/actions/{id}/confirm", "mcp", "Confirms and runs a pending agent action", "200"),
    ("POST", "/mcp/actions/{id}/dismiss", "mcp", "Dismisses a pending agent action", "204"),
];

fn method(name: &str) -> HttpMethod {
    match name {
        "POST" => HttpMethod::Post,
        "PATCH" => HttpMethod::Patch,
        "DELETE" => HttpMethod::Delete,
        _ => HttpMethod::Get,
    }
}

/// The OpenAPI document of every route in [`ROUTES`].
pub fn document() -> OpenApi {
    let paths = ROUTES.iter().fold(PathsBuilder::new(), |paths, (verb, path, tag, summary, status)| {
        let operation = OperationBuilder::new()
            .tag(*tag)
            .summary(Some(*summary))
            .response(*status, ResponseBuilder::new().description(*summary).build());
        paths.path(*path, PathItem::new(method(verb), operation))
    });
    let info = InfoBuilder::new().title("Docs API").version(crate::version::VERSION).build();
    OpenApiBuilder::new().info(info).paths(paths.build()).build()
}

/// The page's title, description and servers.
pub fn config() -> tronc::apiref::Config {
    tronc::apiref::Config {
        title: "Docs API".to_owned(),
        version: crate::version::VERSION.trim_start_matches('v').to_owned(),
        description: "Collaborative documents: a page tree per space, live editing over y-sync, agents over MCP."
            .to_owned(),
        servers: vec!["/api".to_owned()],
        script_url: String::new(),
    }
}
