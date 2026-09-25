#!/bin/sh
# package-runtime.sh - build distributable tarballs of the vendored runtime
# binaries (MySQL, PHP-FPM, nginx, Deno) plus a SHA256SUMS manifest.
#
# The ./osede control binary downloads these tarballs on first run (see
# `./osede fetch`). Upload the generated files as GitHub release assets so
# the repo itself stays small.
#
# Tarballs may carry a glibc tag so the control binary can pick the build
# matching the host it runs on, e.g.
#   runtime-php-glibc231.tar.gz  (built in an Ubuntu 20.04 container)
#   runtime-php-glibc239.tar.gz  (built on a modern host)
# Untagged files (e.g. runtime-mysql.tar.gz) are used when no tagged variant
# fits - suitable for official release builds that run everywhere.
#
# Usage:
#   ./scripts/package-runtime.sh                            # all components, untagged
#   GLIBC_TAG=239 COMPONENTS="php nginx" ./scripts/package-runtime.sh
#   DIST_DIR=/some/path ./scripts/package-runtime.sh
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
DIST="${DIST_DIR:-$ROOT/dist}"
COMPS="${COMPONENTS:-mysql php nginx deno}"
mkdir -p "$DIST"

for comp in $COMPS; do
  if [ ! -d "$ROOT/runtime/$comp" ]; then
    echo "error: runtime/$comp not found (run the environment once first)" >&2
    exit 1
  fi
  if [ -n "${GLIBC_TAG:-}" ]; then
    out="$DIST/runtime-$comp-glibc$GLIBC_TAG.tar.gz"
  else
    out="$DIST/runtime-$comp.tar.gz"
  fi
  echo "packaging runtime/$comp -> $out"
  tar -C "$ROOT/runtime" -czf "$out" "$comp"
done

cd "$DIST"
tmp_sums="$DIST/.sums.new"
for comp in $COMPS; do
  if [ -n "${GLIBC_TAG:-}" ]; then
    f="runtime-$comp-glibc$GLIBC_TAG.tar.gz"
  else
    f="runtime-$comp.tar.gz"
  fi
  sha256sum "$f"
done > "$tmp_sums"

# merge into the existing manifest, replacing entries for the same file name
if [ -f SHA256SUMS ]; then
  cut -d' ' -f2- "$tmp_sums" > "$DIST/.names"
  { grep -v -F -f "$DIST/.names" SHA256SUMS || true; cat "$tmp_sums"; } \
      | sort -k2 > "$DIST/.sums.merged"
  mv "$DIST/.sums.merged" SHA256SUMS
  rm -f "$DIST/.names"
else
  sort -k2 "$tmp_sums" > SHA256SUMS
fi
rm -f "$tmp_sums"

cat SHA256SUMS
echo
echo "done. upload these files as release assets (e.g. release v1.0.0):"
ls -lh runtime-*.tar.gz SHA256SUMS
