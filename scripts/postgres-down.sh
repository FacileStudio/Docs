#!/usr/bin/env sh
set -eu
container="docs_dev_postgres"
if [ "$(docker inspect -f '{{.State.Running}}' "$container" 2>/dev/null || echo missing)" = "true" ]; then
  docker stop "$container" >/dev/null
  echo "stopped docs_dev_postgres"
else
  echo "docs_dev_postgres is not running"
fi
