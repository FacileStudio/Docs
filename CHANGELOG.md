# Changelog

All notable changes to Docs. Versions are local tags until the owner pushes the repo.

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
