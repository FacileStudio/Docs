#!/usr/bin/env sh
set -eu

cd "$(git rev-parse --show-toplevel)"

: "${TEST_DATABASE_URL:=postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT:-5654}/postgres?sslmode=disable}"
export TEST_DATABASE_URL
if ! pg_isready -d "$TEST_DATABASE_URL" -q 2>/dev/null; then
  echo "check: no Postgres at TEST_DATABASE_URL; the database tests will skip (run: mise run db-up)" >&2
fi

echo "==> apps/api"
(
  cd apps/api
  cargo fmt --check
  cargo clippy --all-targets --locked -- -D warnings
  cargo test --locked
)

if [ -f apps/client/package.json ]; then
  echo "==> apps/client"
  (
    cd apps/client
    [ -d node_modules ] || bun install --frozen-lockfile >/dev/null
    bun run check
    bun run test
  )
else
  echo "check: no apps/client on this branch, skipping the client" >&2
fi
