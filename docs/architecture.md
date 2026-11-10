# Docs — Architecture

How a request, an edit and an agent operation move through the Docs API, and what it stores.

```
Internet ──▶ Traefik ──▶ api (:4000, one replica) ─┬──▶ /health, /ready, /version, /docs
                                                   ├──▶ /mcp            tronc::mcp + porte mcpauth
                                                   ├──▶ /api/*          modules
                                                   │     └─ /api/documents/{id}/ws ──▶ room (yrs)
                                                   └──▶ everything else the SPA (apps/client/build)
                                                              │
                                                  Postgres 16 ┴ DATA_DIR/attachments (volume)
```

## Components

| Path | Role |
|---|---|
| `src/server.rs` | Boot in the order of the plan: env, logger, pool, `migrate`, migrations, kits, loops, router |
| `src/auth/` | porte wiring: sessions, Registre sign-in, the spaces store, directory apply, GDPR erase |
| `src/modules/documents/` | The tree, content, trash, revisions, and the one `access()` function |
| `src/modules/rooms/` | One in-memory Yjs room per open page, the y-sync WebSocket |
| `src/modules/search/` | Full-text search, scope-filtered in the same statement |
| `src/modules/attachments/` | Content-addressed files, byte sniffing, the daily sweep |
| `src/modules/share/` | Share links and the public, rate-limited routes |
| `src/modules/export/` | Facile Markdown, CommonMark and PDF exports |
| `src/modules/pending/` | The confirm page's API for MCP actions |
| `src/mcp/` | The seven MCP tools |
| `src/outline/` | `api import-outline` |
| `src/apiref.rs` | The OpenAPI document served at `/docs` |

## Access

Every document route calls `access::access(state, user, document)`:

- a personal page (`space_id IS NULL`) is `manager` for its owner and invisible to everyone else;
- a space page is `editor` for a `member`, `manager` for an `admin` or `owner`, invisible otherwise;
- an instance admin gets no bypass, and invisible always answers `404`, never `403`.

A space member is matched by Registre subject: `space_members.subject` joins
`porte_identities.subject` where `provider` is `OIDC_ISSUER`. Every `Document` answer carries
the caller's `access`. A trashed page opens read-only in its room.

## Collaboration

A page open in a browser is a room: a tokio task that owns a `yrs::Doc` whose `default`
fragment is the y-prosemirror layout Tiptap writes. Browsers speak the standard y-sync and
awareness protocol over `y-websocket`; no custom message exists.

1. On the first join the room loads `documents.ydoc`, or builds it from `content` with
   `encre::yjs::to_update` when it is empty.
2. Each update from an editor is applied, then the fragment is read back with `encre::yjs::read`
   and validated. A failure restores the exact prior state (a full state snapshot taken before
   the update) and closes the sender with 1008. A document over 5 MiB closes it with 1009, as
   does a frame over 2 MiB.
3. A viewer's updates are dropped; its awareness is relayed.
4. Two seconds after the last change, and at once when the last member leaves, the room writes
   `ydoc`, `content`, `text`, `version + 1`, `updated_at` and `updated_by` in one statement. A
   revision is written at most every ten minutes, and when the last member leaves after a change.
5. `POST /ops`, MCP edits and revision restores run inside the room as one Yjs transaction
   (`encre::ops::apply_yjs` or `encre::yjs::write`) and persist before they answer, so open
   editors see them live and other carets stay put.
6. A room unloads 60 seconds after its last member leaves. Trashing, restoring or deleting a
   page closes its room with 1001; clients reconnect to the new state. A directory snapshot that
   removes someone from a space closes their connections to that space's pages with 1008.

Rooms are in memory, so the API runs exactly one replica.

## Data model

| Table | Holds |
|---|---|
| `porte_*` | Accounts, identities, sessions (`0001_porte.sql`, porte's schema verbatim) |
| `spaces`, `space_members` | Registre's directory, replaced by each snapshot; `deleted_at` marks a retired space |
| `directory_state`, `deletion_state` | The snapshot version and the deletion cursor |
| `documents` | The tree (`parent_id`, `position`), `content` JSONB, `text`, `ydoc`, `version`, `search` |
| `document_revisions` | Snapshots of `content` |
| `attachments` | Rows naming files under `DATA_DIR/attachments/<2 hex>/<sha256>` |
| `share_links` | Hashed tokens, `children`, `revoked_at` |
| `mcp_actions` | Pending confirmations of `docs_replace` |

A space missing from a directory snapshot is never deleted: it gets `deleted_at`, loses its
members (their sockets close) and its live pages go to the trash. A space that comes back is
undeleted and its pages stay in the trash for a manager to restore. An empty snapshot while
live spaces exist is refused and logged. Only the hourly purge, for pages trashed more than 30
days ago, or a manager's explicit delete removes a page for good.

A collection from Outline is a top-level page. A page and its subtree share one scope; a move
across scopes is refused.

## Authentication

porte sessions: the `session` cookie (with `X-Facile-CSRF` on mutations) or `Authorization:
Bearer`. Sign-in is Registre OIDC at `/api/auth/oidc`. The WebSocket upgrade checks `Origin`
with `Manager::check_origin`. `/mcp` takes Registre access tokens for the resource
`MCP_RESOURCE`, verified by `porte::mcpauth` behind `tronc::mcp`.

## Cross-app integration

Registre pushes nudges to `/api/registre/sync` and signed deletions to
`/api/registre/deletions`; the app pulls the directory snapshot and the deletion feed. Journal
receives logs when configured. Docs emits no Antenne event in v1.
