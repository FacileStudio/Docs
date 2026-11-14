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
 "title": "", "icon": "", "schema_version": 2, "version": 3,
 "created_at": "RFC 3339", "updated_at": "RFC 3339", "trashed_at": "RFC 3339|null",
 "access": "viewer|editor|manager", "content": {"type": "doc", "content": []}}
```

With `?format=markdown`, `markdown` (Facile Markdown) replaces `content`. `version` grows by one
on every content change; `replace_all` checks it.

`schema_version` is the encre schema the stored `content` was written in. Every page Docs creates
or rewrites is stored at encre's current schema: 3 since encre v0.11.0 (`embed` and `file`
blocks), 2 since v0.8.0 (highlight, badges and status dots, diagrams, columns, stat subtitles).
An older row keeps its `content` and `schema_version` byte for byte until its next write; encre
migrates 1 and 2 to 3 as the identity, so a schema 3 reader reads it as is.

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
| `GET /auth/session` | optional | `{"user":{"email","name"?,"avatar_url"?}|null,"bulle_url"}`, always 200. `bulle_url` is `BULLE_URL`, the origin the client frames the Bulle sidebar from |
| `POST /registre/sync` | none | 202: Registre's nudge; the app pulls the snapshot |
| `POST /registre/deletions` | Registre-signed | 202 once erased; the subject then cannot sign in or get an account again |

## Spaces and documents

| Method and path | Body or query | Answer | Access |
|---|---|---|---|
| `GET /spaces` | | `{"spaces":[{"id","name","role"}]}` | signed in |
| `GET /documents` | `?space_id=&trashed=false` | `[TreeEntry]`, parents first then position | member of the scope |
| `POST /documents` | `{"title","space_id"?,"parent_id"?,"markdown"?}` or `{…,"content"}` | 201 `Document` | editor of the parent or the scope |
| `POST /documents/import-openui` | `{"source","title","space_id"?,"parent_id"?}` | 201 `Document` | editor of the parent or the scope |
| `GET /documents/{id}` | `?format=json|markdown` | `Document` | viewer |
| `PATCH /documents/{id}` | `{"title"?,"icon"?}` (title ≤ 200, icon ≤ 32 characters) | `Document` | editor |
| `POST /documents/{id}/ops` | `{"ops":[Op]}`, at most 100, all or nothing | `{"version","markdown","saved"}`; `saved: false` is applied and live, stored later: never replay it | editor |
| `POST /documents/{id}/move` | `{"parent_id": uuid|null, "before": uuid|null}` | `Document`, `position` the midpoint of its new neighbours | manager |
| `DELETE /documents/{id}` | | 204, the subtree goes to the trash; purged after 30 days | manager |
| `POST /documents/{id}/restore` | | `Document`; to the root when its parent stays trashed | manager |
| `DELETE /documents/{id}/permanent` | | 204, only when trashed (else 412) | manager |
| `GET /documents/{id}/revisions` | | `[{"id","title","schema_version","created_by","author","created_at"}]`, newest 50, no bodies | viewer |
| `GET /documents/{id}/revisions/{rev}` | | one revision with its `content` | viewer |
| `POST /documents/{id}/revisions/{rev}/restore` | | `Document` | editor |
| `GET /search` | `?q=&space_id=` | `[{"id","title","snippet","updated_at"}]`, max 50, `<b>` marks the match | member of the scope |

A missing `space_id` means the caller's personal pages. `Op` is encre's JSON form, for example
`{"op":"insert_after","block_id":"a1b2c3d4e5","markdown":"…"}`; see encre `spec/FORMAT.md`
section 4. A stale `replace_all` is 412, any other refused operation 400 with a message that
says how to narrow it. A `patch` that does not match exactly `expect` times says how many it found
(`patch expected 1 match, found 25: …`) and lists the first 20 with their block ids, then
`and N more`. A move across spaces, or into the page's own subtree, is 400.

### Markdown limits

Every markdown body the API parses is refused with 413 `resource_exhausted` above
`MARKDOWN_MAX_BYTES` (1 MiB by default), before anything is read or written: `markdown` on
`POST /documents`, `source` on `POST /documents/import-openui`, the markdown of one `ops` request
summed over its operations (a `patch` counts `replace` once per `expect`), and the `markdown` of
`docs_create`, `docs_edit` and `docs_replace`. encre
measured up to 0.6 s of CPU for a hostile 1 MiB body; parsing runs on a blocking thread, off the
request workers. JSON bodies are also capped at 1 MiB by tronc, so a larger cap only helps MCP
(4 MiB).

Markdown that nests past encre's limits (`spec/FORMAT.md` rules 8 and 9 and section 2.6: more
than 64 levels of blocks and spans, more than 48 in the document, more than 8 `callout`,
`columns` and `column` on one path) is 400 `invalid_argument`. On a create the message places
it: `markdown at line 3, column 1: …`. An operation answers encre's `OpError` message and changes
nothing.

## Exports

| Method and path | Answer | Access |
|---|---|---|
| `GET /documents/{id}/export.md` | Facile Markdown, `text/markdown; charset=utf-8`, attachment | viewer |
| `GET /documents/{id}/export.commonmark` | portable CommonMark | viewer |
| `GET /documents/{id}/export.pdf` | `application/pdf`; images the caller may see are embedded; 30 s timeout (503) | viewer |

## Attachments

| Method and path | Body | Answer | Access |
|---|---|---|---|
| `POST /documents/{id}/attachments` | multipart `file`: an image or PDF ≤ 20 MiB, any other file ≤ `ATTACHMENT_MAX_BYTES` (25 MiB) | 201 `{"id","url","src","name","size","mime"}`, `src` = `url` = `/api/attachments/{id}` | editor |
| `GET /attachments/{id}` | `?share=<token>` optional | the bytes, `Content-Security-Policy: sandbox`, `X-Content-Type-Options: nosniff` | viewer of its page, or a share token covering it |

PNG, JPEG, GIF, WebP and PDF are typed by their magic number and served inline. Any other file
keeps the `Content-Type` of its multipart part, lowercased and without parameters; a malformed
type, HTML, XML, SVG, JavaScript, or an inline type the bytes do not have is stored as
`application/octet-stream`. Those files are served with
`Content-Disposition: attachment; filename*=UTF-8''<percent-encoded name>`, never inline.

## Embeds and the assistant

| Method and path | Answer | Access |
|---|---|---|
| `GET /embeds/resolve?url=<https link>` | `{"title","description","image","site","favicon"}`; `title` is the host when the page says nothing | session |
| `GET /assistant` | `{"name","accent"}` of the person's Bulle assistant; `{"name":"Bulle","accent":null}` on any failure, never 402 | session |

`/embeds/resolve` reads Open Graph, Twitter and `<title>` tags. The link must be `https`, at most
2048 characters, without credentials, and not a private, loopback, link-local or metadata IP
literal (400 otherwise). The fetch resolves names to public addresses only, follows at most 3
redirects that pass the same checks, stops after 5 s and reads at most 1 MiB of HTML. A failed
fetch answers the bare host. `image` and `favicon` are `https` only. One answer per URL is
reused for 5 minutes. `/assistant` exchanges the session for a Bulle token (`bulle.read`) through
Registre and calls Bulle's `GET /api/assistant`.

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
| 1008 | an update broke the schema or its nesting limits (it was reverted), a malformed frame, or access revoked |
| 1009 | a frame over 2 MiB, or an update that takes the document over 5 MiB |
| 1011 | the room could not load |
| 1013 | the client could not keep up with broadcasts |

A viewer's (or a trashed page's) updates are dropped; its awareness is relayed. A close reason is at most 123 bytes, cut on a character boundary.

## Agent actions (confirm page)

| Method and path | Answer | Access |
|---|---|---|
| `GET /mcp/pending` | `{"actions":[{"id","kind","target","summary","client_id","created_at","expires_at","state"}],"truncated"?}`, newest first, at most 100 | own browser session |
| `POST /mcp/pending/{id}/confirm` | `{"status":"done"}` | own browser session |
| `POST /mcp/pending/{id}/dismiss` | `{"status":"dismissed"}` | own browser session |

These are tronc-rs `PendingPage` on Go's paths. Only the person's own browser sign-in passes
(porte-rs `require_own_session`): a bearer of any kind, a named API token or a CLI, device or
mobile token sent as the session cookie, and a cookie minted before porte-rs 0.8.0 all get 403
`permission_denied`; the person signs in again in the browser. MCP off answers 404. Pending
actions older than 30 days after they closed are pruned every hour, and an account erasure
deletes the person's actions.

## MCP

`POST /mcp` (root, outside CSRF), stateless Streamable HTTP through `tronc::mcp`, mounted when
`MCP_RESOURCE` is set. Registre access tokens for that resource, scopes `docs.read` and
`docs.write`; metadata at `/.well-known/oauth-protected-resource/mcp`. Never plan-gated. Every
description embeds encre's Facile Markdown and operations short form (`src/mcp/format.rs`, from
encre v0.11.0 `docs/agents.md`: highlight, badges, status, GFM alerts, diagram fences, columns, plus the `embed` and `file` fences)
and the limits above. A refusal reaches the agent as an `isError` result with the same message.

| Tool | Scope | Arguments | Answer |
|---|---|---|---|
| `docs_search` | read | `query`, `space_id`? | `{"results":[{"id","title","snippet"}]}` |
| `docs_list_tree` | read | `space_id`?, `cursor`?, `limit`? (1 to 200, 50) | `{"pages":[{"id","parent_id","title"}],"next_cursor"?,"total"}` |
| `docs_list_trash` | read | `space_id`?, `cursor`?, `limit`? (1 to 200, 50) | `{"pages":[{"id","parent_id","title","trashed_at"}],"next_cursor"?,"total"}` |
| `docs_read` | read | `document_id` | `{"id","title","version","markdown","truncated"}` |
| `docs_read_blocks` | read | `document_id`, `range`? | `{"version","blocks":[{"id","type","markdown"}],"truncated","next_range"?}`, cut at the result budget; pass `next_range` as `range` for the rest |
| `docs_create` | write | `title`, `markdown`, `space_id`?, `parent_id`? | `{"id","version","markdown","saved"}` |
| `docs_edit` | write, destructive | `document_id`, `ops` (no `replace_all`) | `{"id","version","markdown","saved"}` |
| `docs_replace` | write, destructive | `document_id`, `markdown`, `expect_version` | `{"status","confirm_url"?}` |
| `docs_update` | write, destructive, idempotent | `document_id`, `title`?, `icon`? | `{"id","parent_id","title","icon","version","trashed"}` |
| `docs_move` | write, destructive, idempotent | `document_id`, `parent_id`?, `before_id`? | the same |
| `docs_trash` | write, destructive, idempotent | `document_id` | the same |
| `docs_restore` | write, idempotent | `document_id` | the same |

`docs_replace` asks the person through MRTR elicitation when the client can; otherwise it
answers `pending_confirmation` with a `confirm_url` to `https://<host>/pending?action=<id>`.
Edits run in the page's room, so open editors see them live. `docs_trash` is the delete tool:
the page and its subtree stay in the trash for 30 days, and `docs_restore` brings them back; a
page that is not in the trash is answered unchanged. Only the REST API deletes for good. Every
tool authorizes through `modules/documents/access.rs`, as the REST routes do. A list cursor is
the opaque `next_cursor` of the previous call.

## Command line

```sh
api migrate status|up
api healthcheck
api ops <document-id> <ops.json | ->
```

`api ops` applies a JSON list of the operations above to one page through its room, with no
author, and prints the `/ops` answer; `[{"op": "read"}]` only reads. In production run it with
`docker exec -i docs-api /api ops <id> - < ops.json`. The room lives in that process, so run it
while nobody has the page open: a room the server holds open would overwrite the change on its
next save.

Back to the [documentation index](README.md).
