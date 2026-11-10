#!/usr/bin/env sh
set -eu
version="$(sed -n 's/^[[:space:]]*"version":[[:space:]]*"\([^"]*\)".*/v\1/p' "${1:-apps/client/package.json}" | head -n 1)"
[ -n "$version" ] || { echo "version: no version in ${1:-apps/client/package.json}" >&2; exit 1; }
echo "$version"
