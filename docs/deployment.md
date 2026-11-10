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

## Bumping a library

`tronc-rs`, `porte-rs` and `encre` come from their public GitHub repos, pinned by tag. To move
one, change its `tag` in `apps/api/Cargo.toml` (tronc appears twice, porte pins its own tronc:
keep them on the same tag so `cargo tree` shows one copy), then run `cargo update -p <crate>`
and commit `Cargo.lock`. An encre bump also changes `@facile/encre` in
`apps/client/package.json` to `github:FacileStudio/encre#<tag>`, followed by `bun install`.
Run `sh scripts/check.sh` before the commit.

## Registering in the suite

Follow plan section 2.10: add the `Docs` client to `Registre/seed.yaml` (redirect
`https://docs-next.facile.studio/api/auth/oidc/callback`, sync `/api/registre/sync`, deletions
`/api/registre/deletions`, back-channel logout `/api/auth/backchannel-logout`, MCP resource
`https://docs-next.facile.studio/mcp` with scopes `docs.read docs.write`), set the Dokploy
environment with the `dokploy-env-safe` flow, deploy, verify, then run
`api import-outline --dry-run` against Outline.

Back to the [documentation index](README.md).
