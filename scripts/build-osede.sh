#!/bin/sh
# build-osede.sh - rebuild the osede control binary (Rust) and install it
# at the project root. Only needed if control/src/main.rs was changed;
# the prebuilt ./osede binary is committed and works out of the box.
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT/control"
# Remap the local rust-src path so the built binary does not embed
# machine-specific absolute paths (e.g. $HOME/.rustup/...).
RUSTSRC="$(rustc --print sysroot)/lib/rustlib/src/rust/library"
if [ -d "$RUSTSRC" ]; then
  RUSTFLAGS="--remap-path-prefix=$RUSTSRC=library"
fi
cargo build --release
cp target/release/osede "$ROOT/osede"
chmod +x "$ROOT/osede"
echo "built $ROOT/osede"
