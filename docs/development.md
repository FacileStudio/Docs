# Docs — Development

How to run the API locally, test it, and pass the gate before a commit.

## Prerequisites

- Rust 1.98.1 (`apps/api/rust-toolchain.toml`) with `rustfmt` and `clippy`
- Docker, for Postgres 16, and `pg_isready` (psql 16 or later)
- bun 1.4 for the client, `filet` 0.18 or later, `mise` and `lefthook`
- Sibling checkouts of `tronc-rs`, `porte-rs` and `encre` under `~/Code/Facile` with their tags
  (`v0.3.0`, `v0.4.0`, `v0.5.0`): until they are pushed, `Cargo.toml` reads them through
  `file://` git URLs

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
| `outline_import.rs` | The importer against a fake Outline serving `tests/testdata/outline` |

Without `TEST_DATABASE_URL` the database tests print a skip banner and pass: read the output.

## The gate

```sh
sh scripts/check.sh
filet check .
```

`check.sh` runs `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and
`cargo test --locked` in `apps/api`, then `bun run check` and `bun run test` in `apps/client`
when it exists. `lefthook` runs it on pre-push. Never raise a filet limit to pass.

Back to the [documentation index](README.md).
