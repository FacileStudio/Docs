# Changelog

All notable changes to Docs. Versions follow SemVer.

## [Unreleased]

## [0.10.0] - 2026-11-14

### Added

- A signed-out visitor on `/` sees a landing page (muse `LandingHeader` and `LandingFooter`, four
  features, two "Se connecter avec Facile" buttons) instead of the sign-in screen. Every other
  path still shows the sign-in screen
- The landing hero plays a mise-en-scene demo: Léa joins a page, her caret types a line, then
  the visitor adds one. Phones get a compact layout without the tree

### Changed

- muse v0.23.0 and mise-en-scene v0.2.1. The client no longer sets its own `font-family`, so
  Magnetik from muse applies everywhere

## [0.9.3] - 2026-11-14

### Added

- The client loads Vision's tracker (`https://vision.facile.studio/s.js`, with `data-mask`) on
  `docs.facile.studio` only, and not on a share page (`/s/<token>`), whose path is the share
  token. The API adds `https://vision.facile.studio` to `script-src`; the loader's hash comes
  with the other inline scripts of the build

## [0.9.2] - 2026-11-13

### Changed

- encre v0.12.0 (still schema 3): every `/` entry has an icon, the Toile, Feuilles, Vision, Docs
  and Nuage embed entries show the app logo (`/toile`, `/feuilles`, ...), the arrow keys scroll
  the `/` menu, and a click outside, a blur or a scroll closes it and keeps the typed text
- The right-click "Transformer en" entries use encre's block icons

## [0.9.1] - 2026-11-13

### Changed

- muse v0.21.2: every icon-only control shows a muse tooltip, including the tree's fold toggle,
  the page icon chooser and, on narrow screens, the Bulle and history buttons
- Every entry of the right-click menus has an icon, "Transformer en" included
- Up, Down, Home and End move through a menu and scroll the active entry into view; disabled
  entries are skipped and Escape gives focus back to where the menu opened
- A menu opened with the mouse no longer draws a focus ring on its first entry; the ring shows
  only when the keyboard moves through it

### Removed

- The Docs close button above the Bulle sidebar: Bulle's own header closes it. Escape outside the
  editor and the header button still close it too

## [0.9.0] - 2026-11-13

### Added

- Pasting a link creates an embed: Toile boards, Feuilles workbooks and Vision shares as live
  frames, Nuage and other links as cards resolved by `/api/embeds/resolve`
- Any file can be dropped into a page and is stored as a `file` block
- A right-click menu in the editor: cut, copy, paste, turn into, duplicate, delete, copy block
  link and "Demander à <assistant>"
- The Bulle sidebar: a button with the user's Bulle blob opens Bulle's chat beside the page, and
  "Demander à <assistant>" in the bubble or right-click menu sends the selected blocks to it
- muse `Presence` shows the other people on a page, and `ConnectionHint` shows only when the
  page is not live
- MCP tools to rename, move, trash and restore pages
- `GET /api/assistant` returns the user's Bulle assistant name

### Changed

- The editor has no toolbar: formatting comes from `/` and the selection bubble menu
- The page icon has no background tile next to the title or in the tree
- Pages are stored at encre schema 3 (encre v0.11.0), which adds `embed` and `file` blocks
- The CSP `frame-src` allows Toile, Feuilles, Vision and Bulle

### Removed

- The "En ligne" status label

## [0.8.3] - 2026-11-13

### Changed

- Deploy: healthchecks run every 30 s with a 1 s start interval, so a deploy cuts over in a few seconds.

## [0.8.2] - 2026-11-13

### Fixed

- Buttons, `role="button"` elements, the tree toggles, the icon chooser and the home cards show
  a pointer cursor, and a disabled button shows `not-allowed`

## [0.8.1] - 2026-11-13

### Fixed

- A room that opens a page whose stored Yjs state is from an older encre schema migrates it
  inside that same Yjs document and stores it with the current `schema_version`. It used to
  trust the stored Yjs state as is, so a future schema migration would have skipped every page
  that had been opened once
- Tree links and the quick-create card show the muse focus outline when reached by keyboard

## [0.8.0] - 2026-11-12

### Added

- Responsive document card grid on the home page with document preview mockups and hover lifts
- Dynamic grid column tracking with a trailing quick-create card button to fill incomplete rows
- Context menu and actions menu on document cards (open, new sub-page, copy link, and trash)

## [0.7.2] - 2026-11-12

### Added

- Right-click context menu on the sidebar document tree: open, create sub-page, copy link, and
  move to trash, with keyboard navigation and edge clamping

### Security

- encre v0.10.0 on both sides (`tag = "v0.10.0"`, `#v0.10.0`), on typst 0.15.1. A
  `[patch.crates-io]` pin of `citationberg` drops quick-xml 0.38 and its advisories from the
  tree. encre's API is unchanged; a wasm trap in the editor now recovers on its own

## [0.7.1] - 2026-11-12

### Security

- porte-rs v0.7.0 and tronc-rs v0.4.0 (one tronc crate). The deletion receiver now holds
  erased subjects: a sign-in, a bearer provisioning or a profile sync for a person Registre
  erased is refused before the account is written, and an account that raced the erasure is
  erased again on the next push or poll
- A member can no longer rewrite another member's presence. The room tracks which Yjs client
  ids each connection claimed first and drops awareness entries for ids another connection owns
- Evicting members who left a space matches the identity provider as well as the subject

### Fixed

- A create or a move under a page that is being trashed can no longer leave a live page under a
  trashed parent, which the 30-day purge would then delete. The write locks the parent and
  re-checks it in the same statement, and the trash locks the whole subtree before it marks it

### Added

- `api ops <document-id> <ops.json | ->`: applies encre operations to one page through its
  room, for operator fixes in the container (`docker exec -i docs-api /api ops <id> -`)

### Removed

- The unused `SpaceStore` (porte `spaces` store) and its conformance test

## [0.7.0] - 2026-11-11

### Changed

- The Docs logo is the suite catalog's mark (`solar:documents-bold-duotone`) in the rail and
  the mobile header. The custom `docs:logo` glyph is gone

### Fixed

- Favicons, touch and manifest icons, the link preview image and the head metadata come from
  the muse v0.19.0 brand generator, so Docs's tab, title, description and og/twitter tags match
  the suite catalog

## [0.6.3] - 2026-11-11

### Security

- porte-rs v0.5.2 (`tag = "v0.5.2"`): drops `openidconnect` and `rsa` 0.9.10
  (RUSTSEC-2023-0071, the Marvin timing attack). Back-channel logout now answers 503 when
  Registre's JWKS cannot be fetched, so Registre retries, instead of 401
- An account erase from Registre now closes the person's sockets in every room with 1008,
  unloads their deleted personal pages, and drops their pending `mcp_actions`. A room whose row
  was deleted stops instead of retrying the save every 2 s forever, and a save whose author no
  longer exists stores a null author instead of failing on the foreign key, so one erased
  account cannot block the edits of everyone else in a shared room
- Open sockets re-check their session every 30 s: after a logout, a back-channel logout or an
  expiry they close with 1008. A socket opened with a bearer JWT (no session row) reconnects
  after 15 minutes. porte-rs has no logout hook, so the check is periodic
- The container runs as `nonroot` on `gcr.io/distroless/cc-debian12:nonroot`, pinned by
  digest, with a `HEALTHCHECK`. A one-shot `docs-data-owner` compose service hands the
  existing `/data` volume to uid 65532 before the API starts

### Changed

- `GET /documents/{id}/revisions` lists metadata only; `GET /documents/{id}/revisions/{rev}`
  answers one revision with its content
- `POST /documents/{id}/ops` and `docs_edit` answer `saved: false` instead of 503 when an edit
  is applied and live but not stored yet; the room keeps retrying, so it must not be replayed
- A PDF export takes its compile slot before it loads images, loads each image once, and
  refuses more than 64 images or 100 MiB of them with 413
- A room applies each socket frame on a blocking thread, so validating a large page no longer
  stalls the async workers
- An attachment upload that matches a stored file refreshes its modification time, so the
  daily sweep cannot delete it while the new row commits

### Removed

- `api import-outline`: Outline is retired and its pages are in Docs. Its icon names stay
  supported by the client


## [0.6.2] - 2026-11-11

### Security

- encre v0.9.4 in the API and the client (`tag = "v0.9.4"`, `github:FacileStudio/encre#v0.9.4`).
  `encre::yjs::read` recursed without a bound, so one hostile Yjs update nested deep enough
  overflowed the stack and killed `docs-api`. It now answers `YjsError::Invalid`, the room
  reverts the update and closes its sender with 1008. A room test sends a 10 000-deep update
  and checks the refusal, the unchanged page and that the room still serves the next editor
- encre v0.9.4 also bounds OpenUI nesting (`openui::Error::TooDeep`) and normalizes pasted tables

### Fixed

- Page icons draw as icons everywhere: the tree, the home list, the page header, the icon
  chooser, search, the trash and the share page. Outline icon names (`academicCap`,
  `collection`) and Font Awesome names (`folder-closed`, `user-graduate`, `book`) map to muse
  Solar glyphs (`apps/client/src/lib/icons.ts`), emoji stay emoji, and an unknown name draws the
  document icon. The icon chooser offers fifteen glyphs next to the emoji. The Outline import
  keeps the icon name again instead of dropping it, since the client now draws it
- Docs has a favicon: `favicon.svg`, `favicon.ico`, `apple-touch-icon.png`, `icon-192.png`,
  `icon-512.png` and `site.webmanifest`, built from the brand glyph like Sablier's
- Layout: long page titles wrap instead of being cut, the document toolbar keeps its actions
  together (icon-only on phones), "Documents récents" stays on one line, trash rows keep their
  title readable on phones, the sidebar starts collapsed below 1280 px, settings hide the
  document tree, and unknown URLs show a French "Page introuvable" page instead of a bare 404

## [0.6.1] - 2026-11-11

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
