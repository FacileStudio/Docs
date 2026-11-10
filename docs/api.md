# Docs — API

Every route, grouped by module, generated from the routers in `apps/api/src`. The live,
machine-readable reference is served by the app at `GET /docs` (Scalar) and
`GET /docs/openapi.json`; `tests/exports.rs` fails when a mounted route is missing from it.

`/docs` sits at the root, as on every suite app, so the client must not use `/docs` as a page
path. Its pages live at `/d/{id}`, `/s/{token}`, `/trash`, `/settings` and `/pending`.

## Conventions

- Everything below is under `/api` unless it starts with another root.
- Auth is a porte session: the `session` cookie, with `X-Facile-CSRF: <any>` on every mutation,
  or `Authorization: Bearer <session token>`.
- Errors are `{"error":{"code","message"}}` with the suite codes (`invalid_argument` 400,
  `unauthenticated` 401, `permission_denied` 403, `not_found` 404, `failed_precondition` 412,
  `resource_exhausted` 413, `rate_limited` 429, `unavailable` 503, `internal` 500).
- Request bodies refuse unknown fields with 400.
- A document the caller cannot see is always 404. Seen but not allowed is 403.
- Access: `viewer` reads, `editor` edits content and title, `manager` also moves, trashes,
  restores, deletes and shares. Personal pages: the owner is `manager`. Space pages: `member` is
  `editor`, `admin` and `owner` are `manager`. A trashed page edits as read-only.

## Types

`Document`:

```json
{"id": "uuid", "space_id": "string|null", "parent_id": "uuid|null", "position": 1.0,
 "title": "", "icon": "", "schema_version": 1, "version": 3,
 "created_at": "RFC 3339", "updated_at": "RFC 3339", "trashed_at": "RFC 3339|null",
 "access": "viewer|editor|manager", "content": {"type": "doc", "content": []}}
```

With `?format=markdown`, `markdown` (Facile Markdown) replaces `content`. `version` grows by one
on every content change; `replace_all` checks it.

`TreeEntry`: `{"id","parent_id","position","title","icon","updated_at"}`, plus `trashed_at` in
the trash listing.

## Meta and auth

| Method and path | Auth | Answer |
|---|---|---|
| `GET /version` (root) | none | `{"version","encre_schema"}` |
| `GET /health`, `/ready` (root and `/api`) | none | `{"status":"ok"}`, `{"status":"ready"}` or 503 |
| `GET /auth/config` | none | `{"sso_only":true,"oidc_enabled"}` |
| `GET /auth/oidc` | none | 302 to Registre |
| `GET /auth/oidc/callback` | none | 302 to `OIDC_SUCCESS_URL` with the session cookie |
| `POST /auth/oidc/exchange` | none | `{"code"}` → `{"user_id","token"}` |
| `POST /auth/oidc/device/exchange` | none | `{"access_token"}` → `{"user_id","token"}`, with `OIDC_CLI_AUDIENCE` |
| `POST /auth/sync-profile` | session | `{"synced"}` |
| `POST /auth/backchannel-logout` | Registre | form `logout_token` |
| `POST /auth/logout` | session | `{"logged_out":true}` |
| `GET /auth/me` | session | `{"email","name"?,"avatar_url"?}` or 401 |
| `GET /auth/session` | optional | `{"user":{"email","name"?,"avatar_url"?}|null}`, always 200 |
| `POST /registre/sync` | none | 202: Registre's nudge; the app pulls the snapshot |
| `POST /registre/deletions` | Registre-signed | 202 once erased |

## Spaces and documents

| Method and path | Body or query | Answer | Access |
|---|---|---|---|
| `GET /spaces` | | `{"spaces":[{"id","name","role"}]}` | signed in |
| `GET /documents` | `?space_id=&trashed=false` | `[TreeEntry]`, parents first then position | member of the scope |
| `POST /documents` | `{"title","space_id"?,"parent_id"?,"markdown"?}` or `{…,"content"}` | 201 `Document` | editor of the parent or the scope |
| `POST /documents/import-openui` | `{"source","title","space_id"?,"parent_id"?}` | 201 `Document` | editor of the parent or the scope |
| `GET /documents/{id}` | `?format=json|markdown` | `Document` | viewer |
| `PATCH /documents/{id}` | `{"title"?,"icon"?}` (title ≤ 200, icon ≤ 32 characters) | `Document` | editor |
| `POST /documents/{id}/ops` | `{"ops":[Op]}`, at most 100, all or nothing | `{"version","markdown"}` | editor |
| `POST /documents/{id}/move` | `{"parent_id": uuid|null, "before": uuid|null}` | `Document`, `position` the midpoint of its new neighbours | manager |
| `DELETE /documents/{id}` | | 204, the subtree goes to the trash | manager |
| `POST /documents/{id}/restore` | | `Document`; to the root when its parent stays trashed | manager |
| `DELETE /documents/{id}/permanent` | | 204, only when trashed (else 412) | manager |
| `GET /documents/{id}/revisions` | | `[{"id","title","schema_version","content","created_by","author","created_at"}]`, newest 50 | viewer |
| `POST /documents/{id}/revisions/{rev}/restore` | | `Document` | editor |
| `GET /search` | `?q=&space_id=` | `[{"id","title","snippet","updated_at"}]`, max 50, `<b>` marks the match | member of the scope |

A missing `space_id` means the caller's personal pages. `Op` is encre's JSON form, for example
`{"op":"insert_after","block_id":"a1b2c3d4e5","markdown":"…"}`; see encre `spec/FORMAT.md`
section 4. A stale `replace_all` is 412, any other refused operation 400 with a message that
says how to narrow it. A move across spaces, or into the page's own subtree, is 400.

## Exports

| Method and path | Answer | Access |
|---|---|---|
| `GET /documents/{id}/export.md` | Facile Markdown, `text/markdown; charset=utf-8`, attachment | viewer |
| `GET /documents/{id}/export.commonmark` | portable CommonMark | viewer |
| `GET /documents/{id}/export.pdf` | `application/pdf`; images the caller may see are embedded; 30 s timeout (503) | viewer |

## Attachments

| Method and path | Body | Answer | Access |
|---|---|---|---|
| `POST /documents/{id}/attachments` | multipart `file`, ≤ 20 MiB | 201 `{"id","url":"/api/attachments/{id}"}` | editor |
| `GET /attachments/{id}` | `?share=<token>` optional | the bytes, `Content-Security-Policy: sandbox`, `X-Content-Type-Options: nosniff` | viewer of its page, or a share token covering it |

The type is sniffed from the bytes: PNG, JPEG, GIF, WebP and PDF are accepted; SVG and all else
are 400.

## Sharing

| Method and path | Body | Answer | Access |
|---|---|---|---|
| `POST /documents/{id}/share` | `{"children": bool}` | 201 `{"id","url":"/s/<token>"}`, token shown once | manager |
| `GET /documents/{id}/share` | | `[{"id","children","created_at"}]`, never the token | manager |
| `DELETE /share/{id}` | | 204 | manager of its page |
| `GET /public/{token}` | | `{"id","title","icon","content","children":[TreeEntry]}` | none, 2/s per IP, burst 30 |
| `GET /public/{token}/documents/{id}` | | the same shape, `children` empty | none, when the link covers children |

`children` lists the whole live subtree when the link was made with `children: true`. Image
sources pointing at `/api/attachments/{id}` come back with `?share=<token>` appended.

## Collaboration WebSocket

`GET /documents/{id}/ws`, upgraded. Session required, viewer access; a cookie upgrade must send
an `Origin` matching the host (porte `check_origin`), a bearer upgrade need not. The protocol is
the standard y-sync and awareness protocol of `y-websocket` (`yrs::sync::Message`), one message
or more per binary frame, document fragment `default` in the y-prosemirror layout:

1. The server opens with `SyncStep1` (its state vector) and the current awareness states.
2. The client sends `SyncStep1`; the server answers `SyncStep2`. The client answers the
   server's `SyncStep1` with its own `SyncStep2`, which carries any offline edits.
3. `Update` messages flow both ways; the server relays accepted ones to every other member.

| Close code | When |
|---|---|
| 1001 | the room closed: restart, trash, restore or delete; reconnect |
| 1008 | an update broke the schema (it was reverted), a malformed frame, or access revoked |
| 1009 | a frame over 2 MiB, or an update that takes the document over 5 MiB |
| 1011 | the room could not load |
| 1013 | the client could not keep up with broadcasts |

A viewer's (or a trashed page's) updates are dropped; its awareness is relayed.

## Agent actions (confirm page)

| Method and path | Answer | Access |
|---|---|---|
| `GET /mcp/actions` | `[{"id","kind","target","summary","client_id","created_at","expires_at","state"}]` | session, own actions |
| `POST /mcp/actions/{id}/confirm` | `{"status":"done"}` | session, not a delegated token |
| `POST /mcp/actions/{id}/dismiss` | 204 | session |

## MCP

`POST /mcp` (root, outside CSRF), stateless Streamable HTTP through `tronc::mcp`, mounted when
`MCP_RESOURCE` is set. Registre access tokens for that resource, scopes `docs.read` and
`docs.write`; metadata at `/.well-known/oauth-protected-resource/mcp`. Never plan-gated. Every
description embeds encre's Facile Markdown and operations short form (`src/mcp/format.rs`).

| Tool | Scope | Arguments | Answer |
|---|---|---|---|
| `docs_search` | read | `query`, `space_id`? | `{"results":[{"id","title","snippet"}]}` |
| `docs_list_tree` | read | `space_id`? | `{"pages":[{"id","parent_id","title"}]}` |
| `docs_read` | read | `document_id` | `{"id","title","version","markdown","truncated"}` |
| `docs_read_blocks` | read | `document_id`, `range`? | `{"version","blocks":[{"id","type","markdown"}]}` |
| `docs_create` | write | `title`, `markdown`, `space_id`?, `parent_id`? | `{"id","version","markdown"}` |
| `docs_edit` | write | `document_id`, `ops` (no `replace_all`) | `{"id","version","markdown"}` |
| `docs_replace` | write, destructive | `document_id`, `markdown`, `expect_version` | `{"status","confirm_url"?}` |

`docs_replace` asks the person through MRTR elicitation when the client can; otherwise it
answers `pending_confirmation` with a `confirm_url` to `https://<host>/pending?action=<id>`.
Edits run in the page's room, so open editors see them live. There is no delete tool.

## Command line

```sh
api migrate status|up
api healthcheck
api import-outline --url https://docs.facile.studio --token <api token> --space <space id> [--dry-run]
```

Back to the [documentation index](README.md).
