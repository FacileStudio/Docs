# Docs client

The SvelteKit 2 single-page app of Docs: a tree of pages per space, edited together in the `@facile/encre` editor. Svelte 5 runes, Tailwind 4, `@facile/muse` v0.17.1, `adapter-static`, bun. The Rust API serves the built `build/` folder in production.

## Run it

`mise.toml` pins Node 24.22.0 and bun 1.4.2. Node 24 matters: vitest runs jsdom 31, which refuses Node 20.

```sh
bun install
bun run dev          # http://localhost:5173, /api and its websockets proxied to 127.0.0.1:4000
bun run check        # svelte-check, warnings fail
bun run test         # vitest
bun run build        # static site in build/
```

### Against the API

1. Start the API on port 4000 (`apps/api`, see the repo `docs/development.md`).
2. Set the API's `OIDC_SUCCESS_URL` to `http://localhost:5173/` so sign-in lands on the dev server.
3. `bun run dev`. Vite proxies `/api/*` to `http://127.0.0.1:4000` with `ws: true`, so the y-websocket connection to `/api/documents/{id}/ws` goes through the same proxy and carries the session cookie.

### Without the API

`DOCS_MOCK=1 bun run dev` replaces the proxy with an in-memory mock of the HTTP API (`src/lib/dev/`). It has no websocket, so the document page shows "Reconnexion…" and the editor stays empty. The tests use the same mock. Production code never imports `src/lib/dev/`.

## What is covered

| Area | Where |
|---|---|
| API client: relative `/api`, `credentials: 'include'`, `X-Facile-CSRF: 1` on every mutation, the `{"error":{"code","message"}}` envelope thrown as `ApiError` | `src/lib/backend.ts` |
| Layout: muse `SideBar` with `SpaceSwitcher` (hidden when there are no spaces), `UserMenu`, `MobileNav`, sign-in through `AuthScreen` | `src/routes/+layout.svelte` |
| Document tree: drag to move (before, inside, after a row, or to the root), Alt+arrows to move with the keyboard, optimistic positions reverted on failure | `src/lib/tree.svelte.ts`, `src/lib/tree-order.ts`, `src/lib/components/Tree*.svelte` |
| Recent documents, new document, markdown import through `loadEncre` and `parseMarkdown` | `src/routes/+page.svelte`, `src/lib/import.ts` |
| Document page: encre `Editor` over `y-websocket` with carets, encre `Presence` in the document header, title and icon editing, connection state (live, reconnecting, offline), read-only editor for viewers and trashed pages | `src/routes/d/[id]/+page.svelte`, `src/lib/collab.ts`, `src/lib/components/CollabEditor.svelte`, `DocumentHeader.svelte` |
| Images: pasted, dropped or picked from the editor's Image command, uploaded through encre's `onimage` hook to the document's attachments | `src/lib/images.ts` |
| Export menu: PDF, Facile Markdown and CommonMark, all from the API | `src/lib/export.ts` |
| Share dialog: create a link (with or without sub-pages), shown once; list and revoke live links | `src/lib/share.ts`, `ShareDialog.svelte` |
| Search palette (⌘K), `ts_headline` snippets rendered without HTML | `Search.svelte`, `src/lib/snippet.ts` |
| History with restore, trash with restore and permanent delete, settings (`SettingsLayout`, profile, preferences) | `src/routes/d/[id]/history`, `src/routes/trash`, `src/routes/settings/[[section]]` |
| Public share page: encre `Document`, read-only, no app chrome, `noindex`, no cookie sent | `src/routes/s/[token]/+page.svelte` |
| Agent confirmation page for `docs_replace` | `src/routes/pending/+page.svelte`, `src/lib/pending.ts` |

Tests: `backend.test.ts`, `tree-order.test.ts`, `tree.svelte.test.ts` (moves against the mock), `share.test.ts`, `export.test.ts`, `pending.test.ts`, `helpers.test.ts` (close codes among them), and `components/CollabEditor.test.ts`, which mounts the encre editor on a stubbed `WebsocketProvider` and checks the room URL, that a Yjs update reaches the editor, the read-only mode and the teardown.

## Against the API contract

The contract is `docs/api.md` at the repo root, as the API is built. The client follows it exactly:

- **Access.** Every `Document` carries the caller's `access` (`viewer`, `editor`, `manager`); the document, header and history pages read it. Scope-wide actions (new page, drag in the tree, trash buttons) use `session.scopeAccess`, the role-to-access table of `docs/api.md`, since no document answers for a whole scope.
- **Lists.** `/spaces` answers `{"spaces":[…]}`; `/documents`, `/search`, `/revisions` and `/documents/{id}/share` answer bare arrays.
- **Trash.** `GET /api/documents?space_id=&trashed=true`.
- **Move.** Both `parent_id` and `before` are always sent; `null` is the root and the end.
- **Sharing.** The dialog lists the live links of a page (`GET /documents/{id}/share`, never the token) and revokes any of them. A new link's token is shown once.
- **Public pages.** `children` is the whole live subtree as tree entries; the page indents it by `parent_id`. The API already appends `?share=<token>` to attachment images.
- **Exports.** PDF, Facile Markdown and CommonMark all come from the API.
- **WebSocket close codes.** 1001 and 1013 reconnect with y-websocket's backoff (1001 also reloads the page record, to catch a trash or restore). 1008 stops: the page reloads the record, shows "not found or no access" when it is gone, else reopens the room on a fresh `Y.Doc` and says the change was refused. 1009 and 1011 stop with an alert and a reload button. Cross-tab BroadcastChannel sync is off, so the server stays the only peer.
- **Agent confirmations.** `/pending?action=<id>` lists the signed-in person's actions from `/api/mcp/actions` and confirms or dismisses one, with muse `ApprovalCard`. Signing in from it returns to it: porte's `/auth/oidc` takes no return path, so `src/lib/return.ts` keeps the page in `sessionStorage` and the layout restores it once a session exists.
- **No `/docs` page.** `/docs` is the API reference served by the binary; client pages are `/`, `/d/{id}`, `/d/{id}/history`, `/s/{token}`, `/trash`, `/settings` and `/pending`.

## Library gaps

- **encre dependency**: `package.json` reads `github:FacileStudio/encre#v0.7.0`. bun installs it flat beside the app's own packages, so one copy of `yjs`, `svelte` and muse runs without a Vite `dedupe`.
- **muse**: `SideBar` takes no app content, so the document tree is a second panel beside the rail on desktop and a `Drawer` on phones. muse bundles no document or share glyph; `src/lib/brand.ts` registers them.
