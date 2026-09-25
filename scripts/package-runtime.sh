#!/bin/sh
# package-runtime.sh - build distributable tarballs of the vendored runtime
# binaries (MySQL, PHP-FPM, nginx, Deno) plus a SHA256SUMS manifest.
#
# The ./osede control binary downloads these tarballs on first run (see
# `./osede fetch`). Upload the generated files as GitHub release assets so
# the repo itself stays small.
#
# Usage:
#   ./scripts/package-runtime.sh            # write to dist/
#   DIST_DIR=/some/path ./scripts/package-runtime.sh
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
DIST="${DIST_DIR:-$ROOT/dist}"
mkdir -p "$DIST"

for comp in mysql php nginx deno; do
  if [ ! -d "$ROOT/runtime/$comp" ]; then
    echo "error: runtime/$comp not found (run the environment once first)" >&2
    exit 1
  fi
  out="$DIST/runtime-$comp.tar.gz"
  echo "packaging runtime/$comp -> $out"
  tar -C "$ROOT/runtime" -czf "$out" "$comp"
done

cd "$DIST"
sha256sum runtime-mysql.tar.gz runtime-php.tar.gz runtime-nginx.tar.gz runtime-deno.tar.gz > SHA256SUMS
cat SHA256SUMS
echo
echo "done. upload these files as release assets (e.g. release v1.0.0):"
ls -lh runtime-*.tar.gz SHA256SUMS
