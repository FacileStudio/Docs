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

echo "==> apps/client"
(
  cd apps/client
  [ -d node_modules ] || bun install --frozen-lockfile >/dev/null
  bun run check
  bun run test
  bun run build
)

echo "==> e2e"
if [ -n "${SKIP_E2E:-}" ]; then
  echo "check: SKIP_E2E is set, skipping the browser run" >&2
elif ! pg_isready -d "$TEST_DATABASE_URL" -q 2>/dev/null; then
  echo "check: no Postgres, skipping the browser run (run: mise run db-up, then mise run e2e)" >&2
elif [ -z "$(ls -d "${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}"/chromium_headless_shell-* 2>/dev/null)" ]; then
  echo "check: no Playwright Chromium, skipping the browser run (see docs/development.md#end-to-end)" >&2
else
  E2E_ADMIN_URL="$TEST_DATABASE_URL" sh scripts/e2e.sh
fi
