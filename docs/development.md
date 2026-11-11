# Docs — Development

How to run the API locally, test it, and pass the gate before a commit.

## Prerequisites

- Rust 1.98.1 (`apps/api/rust-toolchain.toml`) with `rustfmt` and `clippy`
- Docker, for Postgres 16, and `pg_isready` (psql 16 or later)
- bun 1.4 for the client, `filet` 0.18 or later, `mise` and `lefthook`
- Network access to GitHub: `tronc-rs`, `porte-rs` and `encre` come from their public repos by
  tag (`v0.3.0`, `v0.5.1`, `v0.7.0`)

## Setup

```sh
mise trust && mise install
mise run db-up
```

`db-up` starts `postgres:16-alpine` as `docs_dev_postgres` on `127.0.0.1:5654` (password
`postgres`) with `fsync` off, which is fine for a disposable local database and makes the tests
several times faster. `mise run db-down` stops it.

## Run

```sh
mise run dev
```

The API listens on `:4000` with `DATA_DIR=.local/data` and no Registre: sign-in routes answer
`oidc_enabled: false`. The client's `bun run dev` proxies `/api` and the WebSocket to it.

## Tests

```sh
export TEST_DATABASE_URL='postgres://postgres:postgres@127.0.0.1:5654/postgres?sslmode=disable'
cd apps/api && cargo test --locked
```

Unit tests sit beside the code. Integration tests in `apps/api/tests/` build the real router
against Postgres through `tronc::testdb`, one schema per test prefix, kept between runs:

| File | Covers |
|---|---|
| `authorization.rs` | Every document route, for each role and the member of another space |
| `scopes.rs` | Lists, creates and search never cross a space; personal pages |
| `documents.rs` | Markdown round trip, ops, trash, move, revisions, search snippets |
| `directory.rs` | porte's spaces conformance suite, snapshot apply, GDPR erase |
| `lifecycle.rs` | Retired and returning spaces, the empty-snapshot refusal, the 30-day purge |
| `attachments.rs`, `share.rs` | Sniffing, SVG refusal, sandbox headers, sweep, links, rate limit |
| `rooms.rs` | Two yrs peers converge after offline edits; schema-breaking update reverted with 1008; viewers; origin; 1009 |
| `durability.rs` | Kills the real binary one second after an edit and proves nothing is lost |
| `mcp.rs` | tronc's MCP conformance kit with `docs_replace` confirmed, and every tool |
| `exports.rs` | PDF text of a page with every component, markdown exports, OpenUI, the API reference drift test |

Without `TEST_DATABASE_URL` the database tests print a skip banner and pass: read the output.

The client's unit tests run with `cd apps/client && bun run test` (vitest, jsdom, the in-memory
mock of `src/lib/dev/`).

## End to end

```sh
mise run e2e
```

`scripts/e2e.sh` builds the debug `api` binary and the test-only `e2e_seed` example, builds the
client, then runs `apps/client/e2e/docs.e2e.ts` with `bun test` and `playwright-core` driving
headless Chromium. No Registre: `e2e/stack.ts` recreates the `docs_e2e` database on the local
Postgres (`E2E_ADMIN_URL`, default port 5654), starts the API on a free port with
`CLIENT_DIR=apps/client/build` and a scratch `DATA_DIR`, then runs `e2e_seed`. The seed writes
alice (owner) and bob (member) of one space and carol (outsider), and mints each a session with
porte's own session store (`porte::session::Manager::issue`); the test puts them in the
`session` cookie. `e2e_seed` is a Cargo example, so the image never contains it and no route
signs anyone in.

The run proves, against the real API and rooms: each member sees the other's typing in under a
second; a callout inserted by one appears for the other; a pasted image uploads and renders for
both; a reload keeps text, callout and image, and the stored markdown holds them; the PDF export
downloads a real PDF; a share link opens the public page with no cookie, image included; carol
gets "Document introuvable" and a 404.

It needs Playwright's Chromium headless shell under `~/.cache/ms-playwright` (revision 1248 for
`playwright-core` 1.64.1): `cd apps/client && bunx playwright-core install chromium-headless-shell`.
`check.sh` runs it last, and skips it with a message when Postgres or that browser is missing, or
when `SKIP_E2E` is set.

## The gate

```sh
sh scripts/check.sh
filet check .
```

`check.sh` runs `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and
`cargo test --locked` in `apps/api`, then `bun run check`, `bun run test` and `bun run build` in
`apps/client`, then the end-to-end run when it can. `lefthook` runs it on pre-push. Never raise a filet limit to pass.

Back to the [documentation index](README.md).
