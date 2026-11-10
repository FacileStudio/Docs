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
| Document page: encre `Editor` over `y-websocket`, encre presence and carets, title and icon editing, connection state (live, reconnecting, offline), read-only editor for viewers and trashed pages | `src/routes/d/[id]/+page.svelte`, `src/lib/collab.ts`, `src/lib/components/CollabEditor.svelte`, `DocumentHeader.svelte` |
| Image paste and drop, uploaded to the document's attachments, inserted with Tiptap's `setImage` | `src/lib/images.ts` |
| Export menu: PDF and markdown from the API, CommonMark from encre-wasm in the browser | `src/lib/export.ts` |
| Share dialog: create a link (with or without sub-pages), shown once, revoke | `src/lib/share.ts`, `ShareDialog.svelte` |
| Search palette (⌘K), `ts_headline` snippets rendered without HTML | `Search.svelte`, `src/lib/snippet.ts` |
| History with restore, trash with restore and permanent delete, settings (`SettingsLayout`, profile, preferences) | `src/routes/d/[id]/history`, `src/routes/trash`, `src/routes/settings/[[section]]` |
| Public share page: encre `Document`, read-only, no app chrome, `noindex`, no cookie sent | `src/routes/s/[token]/+page.svelte` |

Tests: `backend.test.ts`, `tree-order.test.ts`, `tree.svelte.test.ts` (moves against the mock), `share.test.ts`, `export.test.ts`, `helpers.test.ts`, and `components/CollabEditor.test.ts`, which mounts the encre editor on a stubbed `WebsocketProvider` and checks the room URL, that a Yjs update reaches the editor, the read-only mode and the teardown.

## Contract gaps

The plan (section 8.3) is the contract. Where it says nothing, the client does the following, and the API should match it or say otherwise.

1. **`GET /api/auth/session`** is not in the plan. The client reads Capsule's shape: `{"user": {"email","name"?,"avatar_url"?} | null}`. Sign-in is `GET /api/auth/oidc`.
2. **List shapes.** The plan writes bare arrays for `/documents`, `/search` and `/revisions`, and nothing for `/spaces`. The client accepts a bare array or a wrapper (`spaces`, `documents`, `results`, `revisions`), and reads spaces as `{"id","name","role"}` as in Feuilles 7.5.
3. **`Document` has no field list.** The client expects `id, space_id, parent_id, position, title, icon, schema_version, content | markdown, created_at, updated_at, trashed_at`.
4. **The caller's access is not in any answer.** The client derives it from the space role: personal and `admin`/`owner` manage, `member` edits, anything else views. No space role maps to a viewer, so the read-only editor shows only for an unknown role or a trashed page. An `access` field on `Document` would settle it.
5. **No route lists the trash.** The client calls `GET /api/documents?space_id=&trashed=true`, the Feuilles convention.
6. **Move body.** `parent_id` and `before` are both optional in the plan, so "absent" is ambiguous. The client always sends both: `parent_id: null` is the root, `before: null` is the end.
7. **`POST /documents` needs a title.** A new page is created as "Sans titre".
8. **No route lists share links.** A link can be revoked only from the dialog that created it, since its token is shown once.
9. **Public children.** The client reads `children` as `[{"id","title","icon"?}]` and opens a child at `/s/{token}?doc={id}` through `GET /api/public/{token}/documents/{id}`. Grandchildren are reachable only if the API lists them.
10. **Share and attachment URLs.** A share `url` may be a path or absolute; the client makes it absolute. Images in shared pages that point at `/api/attachments/{id}` get `?share={token}` appended.
11. **No CommonMark route.** The portable export is rendered in the browser by encre-wasm.

## Library gaps

- **encre**: `Editor` has no image upload hook. The client catches paste and drop in the capture phase around the editor, uploads, then calls `setImage` on `instance()`. An `onimage?: (file: File) => Promise<string>` prop would remove that.
- **encre**: `Editor` always renders `Presence` above the toolbar when it has a provider, so the presence avatars sit there, not in the document header.
- **encre dependency**: bun cannot install `git+file://…#v0.5.0`, so `package.json` reads `file:/home/yann/Code/Facile/encre` (at tag v0.5.0 when installed). bun then copies encre's own `node_modules`, a second `yjs`, `svelte` and muse among them; `vite.config.ts` dedupes every encre dependency so one copy of each runs. Switch the line to `github:FacileStudio/encre#v0.5.0` once encre is pushed; the Docker build cannot resolve the local path.
- **muse**: `SideBar` takes no app content, so the document tree is a second panel beside the rail on desktop and a `Drawer` on phones. muse bundles no document or share glyph; `src/lib/brand.ts` registers them.
