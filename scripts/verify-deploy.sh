#!/usr/bin/env sh
set -eu

base="https://${1:-docs.facile.studio}"
fails=0

check() {
  if [ "$2" = "$3" ]; then
    printf '  ok    %-40s %s\n' "$1" "$3"
  else
    printf '  FAIL  %-40s %s (want %s)\n' "$1" "$3" "$2"
    fails=$((fails + 1))
  fi
}

code() { curl -s --max-time 15 -o /dev/null -w '%{http_code}' "$@"; }

printf '== %s ==\n' "$base"
check "GET /" 200 "$(code "$base/")"
check "GET /d/unknown (SPA route)" 200 "$(code "$base/d/unknown")"
check "GET missing bundle" 404 "$(code "$base/_app/immutable/absent.OLDHASH.js")"
check "GET /api/auth/config" 200 "$(code "$base/api/auth/config")"
check "GET /api/auth/session" 200 "$(code "$base/api/auth/session")"
check "GET /api/documents (anonymous)" 401 "$(code "$base/api/documents")"
check "GET /api/health" 200 "$(code "$base/api/health")"
check "GET /api/ready" 200 "$(code "$base/api/ready")"
check "GET /docs" 200 "$(code "$base/docs")"
check "POST unknown route" 405 "$(code -X POST "$base/no-such-route")"
[ "$fails" -eq 0 ] && echo "  => OK" || echo "  => $fails failure(s)"
exit "$fails"
