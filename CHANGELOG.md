# Changelog

All notable changes to Docs. Versions are local tags until the owner pushes the repo.

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
