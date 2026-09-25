#!/bin/sh
# build-runtime.sh - rebuild runtime/php and runtime/nginx inside an old-glibc
# container (Ubuntu 20.04 / glibc 2.31 by default) so they also run on old
# lab VMs. MySQL and Deno are official release builds and work as-is.
#
# Usage (from the project root):
#   ./scripts/build-runtime.sh
#   RUNTIME_IMG=debian:11 ./scripts/build-runtime.sh   # different target glibc
#
# Requires podman or docker with network access. After it finishes, rebuild
# the release tarballs with ./scripts/package-runtime.sh.
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
IMG="${RUNTIME_IMG:-ubuntu:20.04}"
CMD=$(command -v podman || command -v docker || {
    echo "error: need podman or docker" >&2
    exit 1
})
echo "building runtime/php + runtime/nginx inside $IMG"
"$CMD" run --rm -v "$ROOT:/work" "$IMG" /bin/bash /work/scripts/runtime-build-in.sh
echo
echo "done. rebuild the release tarballs with: ./scripts/package-runtime.sh"
