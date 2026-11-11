# Docs

Collaborative documents for the Facile suite, replacing Outline at `docs.facile.studio`. Rust API
in `apps/api`, SvelteKit client in `apps/client`, one container.

## Stack

- **API** (`apps/api`): Rust 1.98 (edition 2024), axum 0.8, sqlx 0.9 on Postgres 16, yrs 0.28
- **Libraries**: `tronc` (tronc-rs v0.3.0), `porte` (porte-rs v0.5.2), `encre` v0.9.4, all from
  their public `github.com/FacileStudio` repos by tag (bump: `docs/deployment.md`). Never copy
  their code here; report a gap instead
- **Client** (`apps/client`): SvelteKit 2, Svelte 5 runes, Tailwind 4, `@facile/muse`, `@facile/encre`

## Commands

```sh
mise run db-up              # Postgres 16 on 127.0.0.1:5654 (container docs_dev_postgres)
sh scripts/check.sh         # the gate: fmt, clippy -D warnings, tests, the client, then e2e
mise run e2e                # Playwright + headless Chromium against the real API and Postgres
filet check .               # after the gate, before every commit
mise run dev                # the API on :4000 against the local database
cd apps/api && cargo run -- migrate status
```

The database tests read `TEST_DATABASE_URL` (check.sh defaults it to the local container) and
print a skip banner without it: read the output before trusting a green run.

## Invariants

- **One replica.** Rooms live in memory; two replicas split a room. `docker-compose.yml` pins
  `replicas: 1`. Do not scale it until rooms have a pub/sub layer.
- **Every write validates.** REST and MCP writes go through `content::prepare` (ids, validate,
  compact). Every Yjs update a room applies is checked with `encre::yjs::read` + validate; a
  failing update is reverted to the exact prior state and its sender closed with 1008.
- **Writes go through the room.** `POST /ops`, MCP edits and revision restores run inside the
  document's room (`encre::ops::apply_yjs`), so open editors see them live.
- **One `access()`.** Every document query is authorized by `modules/documents/access.rs`.
  `None` is always 404; search filters by scope in the same SQL statement.
- **encre is pinned** to one tag in `apps/api/Cargo.toml`. A bump re-runs the room and ops
  tests, and a schema version bump needs `encre::migrate` (stored `schema_version`).
- **Migrations** are append-only `NNNN_name.sql`; `0001_porte.sql` is `porte::pg::SCHEMA` verbatim.
- No inline comments, no comments in function bodies; doc comments on public items.
