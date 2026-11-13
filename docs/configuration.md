# Docs — Configuration

Every environment variable the API reads, from `apps/api/src/env.rs`, tronc's `env::Core` and
porte's configuration. A missing required variable, or an inconsistent pair, exits 1 at boot.

| Variable | Required | Default | What it does |
|---|---|---|---|
| `DATABASE_URL` | yes | — | Postgres connection string |
| `PORT` | no | `8080` (tronc) | Listen port. The image sets `4000` |
| `APP_ENV` | no | `development` | `development`, `staging` or `production` |
| `LOG_LEVEL` | no | `info` | `debug`, `info`, `warn`, `error` |
| `TRUSTED_PROXIES` | prod | — | `private,cloudflare` behind Traefik and Cloudflare |
| `CORS_ALLOWED_ORIGINS` | no | — | Empty in production: the client is same-origin |
| `CLIENT_DIR` | no | `./client` | The built SPA. The image sets `/client` |
| `DATA_DIR` | no | `/data` | Attachments live under `DATA_DIR/attachments`. Mount a volume here |
| `JOURNAL_URL`, `JOURNAL_TOKEN` | no | — | Ship logs to Journal when both are set |
| `OIDC_ISSUER` | prod | — | `https://sso.facile.studio`. Turns sign-in, directory sync and deletions on. Production refuses to boot without it |
| `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET` | with issuer | — | The Registre client |
| `OIDC_REDIRECT_URL` | with issuer | — | `https://<host>/api/auth/oidc/callback` |
| `OIDC_SUCCESS_URL` | with issuer | — | `https://<host>/` |
| `OIDC_CLI_AUDIENCE` | no | — | Mounts `POST /api/auth/oidc/device/exchange` for that CLI client |
| `OIDC_MACHINE_AUDIENCE`, `OIDC_SCOPE_PREFIX` | no | — | Accept Registre machine JWTs on the bearer path |
| `PORTE_TOKEN_KEY` | prod | — | Seals stored OIDC tokens. Production refuses to boot without it |
| `MCP_RESOURCE` | no | — | `https://<host>/mcp`. Its presence mounts `/mcp` |
| `MCP_STATE_KEY` | with MCP | — | HMAC key of the confirmation flow; boot fails without it when MCP is on |
| `ATTACHMENT_MAX_BYTES` | no | `26214400` | The largest attachment that is not an image or a PDF (those stay at 20 MiB). A positive byte count |
| `BULLE_URL` | no | `https://bulle.facile.studio` | Bulle's origin, for `GET /api/assistant` |
| `BULLE_CLIENT_ID` | no | `bulle` | Bulle's Registre client id: the audience Docs exchanges the person's token for |
| `MARKDOWN_MAX_BYTES` | no | `1048576` | The largest markdown body one request, MCP call or imported page may ask the API to parse; above it, 413. A positive byte count. See [API, markdown limits](api.md#markdown-limits) |
| `TEST_DATABASE_URL` | tests | — | Postgres the integration tests create their schemas in |
| `APP_VERSION` | build | `dev` | Read at compile time; the Dockerfile stamps it from `apps/client/package.json` |

`SSO_ONLY` is always true: the local password routes are never registered. tronc's `PORT`
default is 8080, as in Go tronc; the image and `mise run dev` set 4000.

## Fixed limits

| Limit | Value | Where |
|---|---|---|
| Image and PDF size | 20 MiB | `modules/attachments/sniff.rs` |
| Inline types | PNG, JPEG, GIF, WebP, PDF, by magic number; any other file is a download | same |
| Link preview | 5 s, 1 MiB of HTML, 3 redirects, reused 5 minutes per URL | `modules/embeds/` |
| Document JSON | 5 MiB | `modules/documents/content.rs` |
| Request JSON body | 1 MiB (tronc), MCP request 4 MiB | `tronc::json`, `tronc::mcp` |
| Markdown nesting | 64 levels in markdown, 48 in the document, 8 `callout`, `columns` and `column` nodes on one path | encre v0.11.0 |
| WebSocket frame | 2 MiB, closed with 1009 | `modules/rooms/ws.rs` |
| Room persist debounce, unload | 2 s, 60 s | `modules/rooms/room.rs` |
| Revisions | one per 10 minutes of activity | `modules/documents/write.rs` |
| Operations per request | 100 | `modules/documents/ops.rs` |
| PDF compile | 30 s, 2 at once | `modules/export/pdf.rs` |
| Public pages | 2 requests a second per IP, burst 30 | `modules/share/router.rs` |
| Search hits | 50 | `modules/search/service.rs` |
| Trash purge | pages trashed over 30 days ago, hourly | `modules/documents/purge.rs` |

Back to the [documentation index](README.md).
