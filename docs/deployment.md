# Docs — Deployment

How Docs ships: one image, one container, one Traefik router, one hostname, one replica.

## Topology

```
docs.facile.studio ──▶ Traefik (web → websecure) ──▶ docs-api:4000 ──▶ docs-db:5432
                                                              │
                                                   volume docs_attachments at /data
```

`/api/*` is the API, `/docs` the API reference, `/mcp` the MCP endpoint, everything else the
SPA. There is no `PathPrefix` and no `stripprefix`. The WebSocket (`/api/documents/{id}/ws`)
goes through the same router; Traefik forwards upgrades without configuration.

## One replica

Rooms are held in memory: two replicas would each hold a different copy of an open page, and
edits made on one would never reach the other. `docker-compose.yml` pins `deploy.replicas: 1`
and `container_name`, so a deploy never runs two containers side by side. A
deploy disconnects every editor for a few seconds; y-websocket reconnects and resends what the
server lacks. This holds until rooms get a pub/sub layer.

## Build

```sh
docker build -t docs .
```

The root `Dockerfile` builds the client with bun, builds the API with `cargo build --release
--locked` (the version comes from `apps/client/package.json` through `scripts/version.sh`, and
the build fails when it cannot be read), and copies both into `gcr.io/distroless/cc-debian12:nonroot`, pinned by digest. The API runs
as `nonroot` (uid 65532). The `/data` volume was created by a root container, so
`docker-compose.yml` runs a one-shot `docs-data-owner` service (busybox) that hands `/data` to
65532 before `docs-api` starts; it is idempotent. The entrypoint is `/api`, so
`docker run docs migrate status` works.

## Migrations

Migrations run at boot, behind sqlx's lock. `0001_porte.sql` is porte's schema verbatim,
`0002_docs.sql` the app tables, `0003_mcp_actions.sql` tronc's pending-action table.

## Health

`/health` and `/ready` (also under `/api`). The compose healthcheck re-execs the binary:
`["CMD", "/api", "healthcheck"]`. After a deploy run `sh scripts/verify-deploy.sh
docs.facile.studio` or the `deploy-verify` flow.

## Bumping a library

`tronc-rs`, `porte-rs` and `encre` come from their public GitHub repos, pinned by tag. To move
one, change its `tag` in `apps/api/Cargo.toml` (tronc appears twice, porte pins its own tronc:
keep them on the same tag so `cargo tree` shows one copy), then run `cargo update -p <crate>`
and commit `Cargo.lock`. An encre bump also changes `@facile/encre` in
`apps/client/package.json` to `github:FacileStudio/encre#<tag>`, followed by `bun install`.
Run `sh scripts/check.sh` before the commit.

## Registering in the suite

Follow plan section 2.10: add the `Docs` client to `Registre/seed.yaml` (redirect
`https://docs.facile.studio/api/auth/oidc/callback`, sync `/api/registre/sync`, deletions
`/api/registre/deletions`, back-channel logout `/api/auth/backchannel-logout`, MCP resource
`https://docs.facile.studio/mcp` with scopes `docs.read docs.write`), set the Dokploy
environment with the `dokploy-env-safe` flow, deploy, verify.

The assistant name in the Bulle sidebar (`GET /api/assistant`) needs Registre to grant the
`Docs` client a `token_exchange` to the `bulle` audience with scope `bulle.read`. Until it does,
the route answers the default name. `BULLE_URL` (default `https://bulle.facile.studio`) and
`BULLE_CLIENT_ID` (default `bulle`) only need setting for another Bulle. The CSP lets the client
frame Toile, Feuilles, Vision, Bulle and Docs itself (`frame-src`, `src/security.rs`).

Back to the [documentation index](README.md).
