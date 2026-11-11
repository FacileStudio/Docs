# Docs

[![ci](https://github.com/FacileStudio/Docs/actions/workflows/ci.yml/badge.svg)](https://github.com/FacileStudio/Docs/actions/workflows/ci.yml)

Collaborative documents for the Facile suite: a tree of pages per space, edited together in
real time, searchable, exportable to PDF, readable and writable by agents.

Docs replaces Outline at `docs.facile.studio`.

## What it does

- A page tree per Registre space, plus personal pages, with drag-to-move, trash and restore
- Live editing in the Tiptap-based `@facile/encre` editor over the standard y-sync protocol
- Every edit checked against the encre schema on the server, broken updates reverted
- Revisions every ten minutes of activity, restorable
- Full-text search that never leaves the caller's scope
- Image and PDF attachments, typed by their bytes and served sandboxed
- Public read-only share links, optionally covering a page's subtree
- Markdown, CommonMark and PDF export, an Outline importer, and seven MCP tools for agents

## Stack

| Layer | Tech |
|---|---|
| API | Rust 1.98, axum 0.8, sqlx 0.9, PostgreSQL 16, yrs 0.28, tronc-rs 0.3, porte-rs 0.4, encre 0.5 |
| Client | SvelteKit 2, Svelte 5 (runes), Tailwind CSS 4, muse (`@facile/muse`), `@facile/encre` |
| Storage | PostgreSQL 16, attachments on a volume at `DATA_DIR` |
| Deploy | Docker Compose, one container and one replica behind Traefik |

## Quick start

```sh
docker compose up -d --build
```

Compose reads the variables of [docs/configuration.md](docs/configuration.md) from a `.env`
beside it.

The image needs the three libraries on GitHub first: see
[docs/deployment.md](docs/deployment.md#before-the-first-deploy).

### Local development

```sh
mise trust && mise install
mise run db-up
mise run dev
```

The API listens on <http://localhost:4000>; run the client from `apps/client` with `bun run
dev`. The API reference is at <http://localhost:4000/docs>.

### Agents

With `MCP_RESOURCE=https://<host>/mcp` set, point an MCP client at that URL and sign in with
Registre. `docs_read_blocks` lists block ids, `docs_edit` changes blocks in place, and
`docs_replace` asks the person before it overwrites a page.

### Importing Outline

```sh
cd apps/api
cargo run --release -- import-outline --url https://docs.facile.studio \
  --token "$OUTLINE_TOKEN" --space <registre space id> --dry-run
```

Drop `--dry-run` to write. The import runs in one transaction.

## Configuration

| Variable | What it does |
|---|---|
| `DATABASE_URL` | Postgres connection string |
| `DATA_DIR` | Where attachments are stored, `/data` in the image |
| `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET` | Registre sign-in and the directory |
| `OIDC_REDIRECT_URL`, `OIDC_SUCCESS_URL` | The callback and landing URLs |
| `PORTE_TOKEN_KEY` | Seals stored OIDC tokens; required in production |
| `MCP_RESOURCE`, `MCP_STATE_KEY` | Turn `/mcp` on |

Full reference: [docs/configuration.md](docs/configuration.md).

## Structure

```
apps/
  api/       Rust API: auth/ (porte), modules/ (documents, rooms, search, attachments,
             share, export, pending), mcp/, outline/, migrations/, tests/
  client/    SvelteKit 5 client
docs/        Architecture, configuration, development, deployment, API
scripts/     check.sh, version.sh, postgres-up.sh, postgres-down.sh, verify-deploy.sh
```

## Documentation

| Doc | What's in it |
|---|---|
| [Architecture](docs/architecture.md) | Request flow, rooms, data model, how the pieces fit |
| [Configuration](docs/configuration.md) | Every environment variable and default |
| [Development](docs/development.md) | Local setup, tests, the quality gate |
| [Deployment](docs/deployment.md) | Docker Compose, Dokploy, Traefik routing, the first deploy |
| [API](docs/api.md) | HTTP endpoints, the WebSocket protocol and the MCP tools |

## License

Docs is licensed under the [GNU Affero General Public License v3.0](LICENSE) (`AGPL-3.0-only`). A commercial licence is available from Facile Studio for use that cannot meet the AGPL's terms.

The page icons in `apps/client/src/lib/icon-data.ts` are [Solar](https://www.figma.com/community/file/1166831539721848736) by 480 Design, under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).

---

Part of the [Facile Suite](https://facile.studio) — self-hosted tools for creative studios
and freelancers. One login, zero cloud dependency.
