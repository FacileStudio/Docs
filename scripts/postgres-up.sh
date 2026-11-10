#!/usr/bin/env sh
set -eu

port="${POSTGRES_PORT:-5654}"
container="docs_dev_postgres"

case "$(docker inspect -f '{{.State.Running}}' "$container" 2>/dev/null || echo missing)" in
true) echo "postgres is already running on 127.0.0.1:${port}" ;;
false) docker start "$container" >/dev/null ;;
*)
  docker run -d --name "$container" -e POSTGRES_PASSWORD=postgres -p "127.0.0.1:${port}:5432" \
    -v docs_dev_pgdata:/var/lib/postgresql/data postgres:16-alpine \
    -c fsync=off -c synchronous_commit=off -c full_page_writes=off >/dev/null
  ;;
esac

attempt=0
until docker exec "$container" pg_isready -U postgres >/dev/null 2>&1; do
  attempt=$((attempt + 1))
  [ "$attempt" -lt 60 ] || { echo "postgres did not become ready in 60s: docker logs ${container}" >&2; exit 1; }
  sleep 1
done
echo "postgres is ready: postgres://postgres:postgres@127.0.0.1:${port}/postgres?sslmode=disable"
