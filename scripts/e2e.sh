#!/usr/bin/env sh
set -eu

cd "$(git rev-parse --show-toplevel)"

if [ -z "$(ls -d "${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}"/chromium_headless_shell-* 2>/dev/null)" ]; then
  echo "e2e: no Playwright Chromium; install it with: cd apps/client && bunx playwright-core install chromium-headless-shell" >&2
  exit 2
fi

(cd apps/api && cargo build --locked --bin api --example e2e_seed)
(
  cd apps/client
  [ -d node_modules ] || bun install --frozen-lockfile >/dev/null
  bun run build
  bun test --timeout 180000 ./e2e/docs.e2e.ts
)
