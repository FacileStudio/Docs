# Docs — Deployment

How Docs ships: one image, one container, one Traefik router, one hostname, one replica.

## Topology

```
docs-next.facile.studio ──▶ Traefik (web → websecure) ──▶ docs-api:4000 ──▶ docs-db:5432
                                                              │
                                                   volume docs_attachments at /data
```

`/api/*` is the API, `/docs` the API reference, `/mcp` the MCP endpoint, everything else the
SPA. There is no `PathPrefix` and no `stripprefix`. The WebSocket (`/api/documents/{id}/ws`)
goes through the same router; Traefik forwards upgrades without configuration.

## One replica

Rooms are held in memory: two replicas would each hold a different copy of an open page, and
edits made on one would never reach the other. `docker-compose.yml` pins `deploy.replicas: 1`
and `update_config.order: stop-first`, so a deploy never runs two containers side by side. A
deploy disconnects every editor for a few seconds; y-websocket reconnects and resends what the
server lacks. This holds until rooms get a pub/sub layer.

## Build

```sh
docker build -t docs .
```

The root `Dockerfile` builds the client with bun, builds the API with `cargo build --release
--locked` (the version comes from `apps/client/package.json` through `scripts/version.sh`, and
the build fails when it cannot be read), and copies both into `gcr.io/distroless/cc-debian12`.
The image runs as root, without `:nonroot`: the named volume at `/data` is owned by root, the
same rule Place gives for apps that write to a volume. The entrypoint is `/api`, so
`docker run docs migrate status` and `docker run docs import-outline …` work.

## Migrations

Migrations run at boot, behind sqlx's lock. `0001_porte.sql` is porte's schema verbatim,
`0002_docs.sql` the app tables, `0003_mcp_actions.sql` tronc's pending-action table.

## Health

`/health` and `/ready` (also under `/api`). The compose healthcheck re-execs the binary:
`["CMD", "/api", "healthcheck"]`. After a deploy run `sh scripts/verify-deploy.sh
docs-next.facile.studio` or the `deploy-verify` flow.

## Before the first deploy

The three libraries are local tags only, read through `file://` URLs that no build server can
reach. Once the owner pushes them, change these lines, then run `cargo update -p tronc -p porte
-p encre` and commit `Cargo.lock`:

| File | From | To |
|---|---|---|
| `apps/api/Cargo.toml` (twice) | `git = "file:///home/yann/Code/Facile/tronc-rs"` | `git = "https://github.com/FacileStudio/tronc-rs"` |
| `apps/api/Cargo.toml` | `git = "file:///home/yann/Code/Facile/porte-rs"` | `git = "https://github.com/FacileStudio/porte-rs"` |
| `apps/api/Cargo.toml` | `git = "file:///home/yann/Code/Facile/encre"` | `git = "https://github.com/FacileStudio/encre"` |
| `apps/client/package.json` | `"@facile/encre": "file:/home/yann/Code/Facile/encre"` | `"@facile/encre": "github:FacileStudio/encre#v0.5.0"` |

porte-rs v0.4.0 itself depends on `tronc-rs` by a `file://` URL at `v0.2.0`; it needs the same
edit and a new porte tag first. While `encre` is private, the API stage needs a build secret
(`RUN --mount=type=secret,id=git_token` with `CARGO_NET_GIT_FETCH_WITH_CLI=true`).

## Registering in the suite

Follow plan section 2.10: add the `Docs` client to `Registre/seed.yaml` (redirect
`https://docs-next.facile.studio/api/auth/oidc/callback`, sync `/api/registre/sync`, deletions
`/api/registre/deletions`, back-channel logout `/api/auth/backchannel-logout`, MCP resource
`https://docs-next.facile.studio/mcp` with scopes `docs.read docs.write`), set the Dokploy
environment with the `dokploy-env-safe` flow, deploy, verify, then run
`api import-outline --dry-run` against Outline.

Back to the [documentation index](README.md).
