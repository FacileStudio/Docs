# Changelog

All notable changes to Docs. Versions follow SemVer.

## [Unreleased]

### Fixed

- Pages imported from Outline carried Outline's icon names (`academicCap`, `collection`) as their
  icon. The tree, the home list, the page header, the trash and the share page printed that name
  as text over the title. The client now draws an icon only when it is an emoji, and the Outline
  import keeps emoji icons only
- The sidebar user card shows the Registre profile picture: every sign-in copies the `https`
  `picture` claim into `porte_users.avatar_url` (porte-rs stores no avatars), and
  `/auth/me` and `/auth/session` fall back to the directory picture of a space member who signed
  in before
- muse's sidebar, user menu and mobile nav speak French, like the rest of Docs, and the menu
  hides the language row the app does not translate

## [0.6.0] - 2026-11-11

### Schema 2

- encre v0.9.0 in the API and the client (`tag = "v0.9.0"`, `github:FacileStudio/encre#v0.9.0`).
  Pages may now hold encre schema 2: `==highlight==`, `[badge:<tone> text]` and `[status:<state>]`,
  `mermaid`, `d2` and `dot` diagrams, `columns` of two to four `column`s, `stat.sub`, and GFM
  alerts (`> [!NOTE]`) read as callouts
- Every page created or rewritten (REST, MCP, room, revision restore, Outline import) stores
  `schema_version = 2`. Schema 1 rows stay as they are until their next write: encre migrates
  1 to 2 as the identity, and a test serves a schema 1 row unchanged
- The Outline import now stores `schema_version` instead of the column default of 1
- The editor and the share page get schema 2 from encre, with `locale` taken from the browser
  (fr, en, es or de; French otherwise) and mermaid diagrams drawn by `mermaid`, imported only
  when a mermaid diagram is shown, with `securityLevel: 'strict'`. d2 and dot show their source.
  No syntax highlighter: code blocks stay encre's escaped text
- MCP tool descriptions carry encre v0.9.0's short form (highlight, badges, status, alerts,
  diagrams, columns) and the size and nesting limits

### Security

- Every markdown body the API parses is capped at `MARKDOWN_MAX_BYTES` (1 MiB by default) with
  413 `resource_exhausted`: create from markdown, `import-openui` source, the summed markdown of
  one `ops` request, each Outline page (the dry run already names oversized pages), and
  `docs_create`, `docs_edit` and `docs_replace`. encre measured up to 0.6 s for a hostile 1 MiB
- Markdown parsing runs on a blocking thread: creates and OpenUI imports through
  `spawn_blocking`, and room transactions (ops, revision restores) with their validation too

### Changed

- Markdown past encre's nesting limits (64 levels in markdown, 48 in the tree, 8 callouts and
  columns on one path) is 400 `invalid_argument`; on a create the message names the line and
  column (`markdown at line 3, column 1: …`). A room update past them is reverted and closed
  with 1008
- A `patch` that matches too often reports the full count and lists the first 20 matches, then
  `and N more`, in the ops answer and the MCP error

### Fixed

- A WebSocket close reason is cut to 123 bytes. A longer one (a validation path of a deeply
  nested update) failed to send, so the client saw 1006 and reconnected instead of reading 1008

## [0.5.2] - 2026-11-10

### Changed

- Licensed under AGPL-3.0-only, like Capsule and Plume.

## [0.5.1] - 2026-11-10

### Changed

- tronc-rs v0.3.0, porte-rs v0.5.1 and encre v0.7.0 come from their public GitHub repos by
  tag; the client reads `github:FacileStudio/encre#v0.7.0`, and the image builds with no
  local paths
- The client's Vite config no longer dedupes encre's dependencies: the GitHub install is flat

### Fixed

- Signing in from `/pending?action=<id>` (or any other page) returns to that page instead of
  the home page: the client keeps the path in `sessionStorage`, since porte's `/auth/oidc`
  takes no return path

## [0.5.0] - 2026-11-10

### Added

- The SvelteKit client, merged from `feat/client`, served by the API from `CLIENT_DIR`
- `/pending?action=<id>`: the signed-in owner confirms or dismisses a `docs_replace` request
  from `/api/mcp/actions`, with muse `ApprovalCard`
- The share dialog lists a page's live links and revokes any of them
- Images upload through encre's `onimage` hook (paste, drop, and the editor's Image command);
  encre `Presence` sits in the document header
- `mise run e2e`: Playwright with headless Chromium against the real API, Postgres and rooms,
  seeded by the test-only `e2e_seed` example with sessions minted through porte's store
- `scripts/check.sh` runs the client's check, tests and build, then the end-to-end run when
  Postgres and Chromium are there; filet requires both apps

### Changed

- The client follows `docs/api.md` as built: `Document.access` replaces the role guessing, exact
  list shapes, server-side CommonMark export, the public subtree as tree entries, share tokens
  on images added by the API only
- WebSocket close codes: 1001 and 1013 reconnect with backoff, 1008 reloads the page (or says
  it is gone), 1009 and 1011 stop with an alert; cross-tab BroadcastChannel sync is off
- porte-rs v0.5.0 (tronc v0.3.0): one tronc crate, porte's MCP verifier used as is, and
  `porte::Config::from_vars`; the app's verifier shim and error conversions are gone
- encre v0.7.0 in the API and the client

## [0.4.0] - 2026-11-10

### Added

- PDF export through `encre::typst` and `encre::pdf`, off the async threads, 30 s timeout, two
  at a time, with the attachments the caller may see embedded and the title in the footer
- Facile Markdown and CommonMark exports
- `POST /api/documents/import-openui`, an OpenUI program saved as a page
- `/mcp` with seven tools; `docs_replace` goes through tronc's confirmation flow, confirmed on
  the client's `/pending` page through `/api/mcp/actions`
- The API reference at `/docs`, checked against the router by a test
- `api import-outline` with `--dry-run`
- The Dockerfile, `docker-compose.yml` (one replica, attachments volume), the docs, CLAUDE.md

### Changed

- encre moved from v0.4.0 to v0.5.0
- A space missing from a directory snapshot is retired (`spaces.deleted_at`) instead of
  deleted: its pages go to the trash, its members lose access and their sockets close, and a
  returning space is undeleted. An empty snapshot while spaces exist is refused
- Pages trashed more than 30 days ago are purged hourly

## [0.3.0] - 2026-11-10

### Added

- Collaboration rooms on yrs with the y-sync and awareness protocol, schema validation of every
  update with an exact revert and 1008, persistence two seconds after a change, revisions,
  size limits (1009), viewer updates dropped, eviction on directory changes
- `POST /ops` and revision restores run through the room, live in open editors

## [0.2.0] - 2026-11-10

### Added

- Attachments: content-addressed files, types sniffed from the bytes, SVG refused, served with
  `Content-Security-Policy: sandbox`, a daily sweep of unreferenced files
- Share links, listed and revocable, and the public pages, rate limited per IP

## [0.1.0] - 2026-11-10

### Added

- The Rust API on tronc-rs and porte-rs: Registre sign-in, directory sync, GDPR erase, the
  spaces store with porte's conformance suite
- Documents: tree, create from markdown or encre JSON, read as JSON or markdown, rename, move,
  trash, restore, permanent delete, revisions, agent operations, full-text search
- One `access()` per document, `access` on every `Document`, 404 for anything out of scope
